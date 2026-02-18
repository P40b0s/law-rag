use std::fmt::Debug;

use rag_core::{Chunk, ChunkMeta, Encoder, ServiceStatus};
use systema_client::{DocumentNode, DocumentNodes};
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

use crate::chunker::Chunker;

/// Упрощённый чанкер: одна нода — один чанк (если помещается).
///
/// Особенности:
/// - Заголовочные ноды (статья/раздел/глава без содержания) пропускаются.
/// - Если непосредственный родитель ноды оканчивается на «:», его текст
///   добавляется в начало чанка, чтобы сохранить контекст перечисления.
pub struct SimpleChunker<C: ToString + Debug + AsRef<str>>(DocumentNodes<C>);

impl<C: ToString + Debug + AsRef<str>> SimpleChunker<C>
{
    pub fn new(nodes: DocumentNodes<C>) -> Self
    {
        Self(nodes)
    }
}

impl<C: ToString + Debug + AsRef<str> + Sync> Chunker for SimpleChunker<C>
{
    async fn get_chunks<'m, E: Encoder + Sync + Send>(
        &self,
        encoder: &E,
        sender: UnboundedSender<ServiceStatus>,
    ) -> anyhow::Result<Vec<Chunk>>
    {
        let hash = self.0.hash();
        let mut chunks = Vec::new();
        let mut chunk_num = 1;

        for node in &self.0
        {
            let content = node.converted_content().as_ref().trim();

            // Пустые ноды (параграф-разделитель и т.п.) пропускаем
            if content.is_empty()
            {
                continue;
            }

            // Заголовочные ноды не несут смысловой нагрузки — пропускаем
            if is_header_node(node)
            {
                continue;
            }

            // Путь вычисляется один раз на ноду, а не на каждый чанк
            let path = self.0.find_all_parents_as_str(node);

            // Если непосредственный родитель — определение/перечисление (оканчивается на «:»),
            // добавляем его текст в начало чанка чтобы не терять контекст
            let parents = self.0.find_all_parents_by_node(node);
            let context_prefix = context_prefix_from_parents(&parents);

            let sub_chunks = split_node_text(content, encoder)?;

            for (idx, (text, token_count)) in sub_chunks.into_iter().enumerate()
            {
                let chunk_content = match &context_prefix
                {
                    Some(prefix) => format!("{}\n{}", prefix, text),
                    None => text,
                };

                chunks.push(Chunk
                {
                    publication_url: self.0.publication_url().to_owned(),
                    document_url: format!("http://actual.pravo.gov.ru/list.html#hash={}", hash),
                    title: self.0.title().to_owned(),
                    number: self.0.number().to_owned(),
                    sign_date: self.0.sign_date().to_owned(),
                    hash: hash.to_owned(),
                    path: path.clone(),
                    links_hashes: node.links_hashes().cloned(),
                    content: chunk_content,
                    embeddings: None,
                    meta: Some(ChunkMeta
                    {
                        chunk_index: idx,
                        token_count,
                    }),
                });

                let _ = sender.send(ServiceStatus::process_chunk(hash, chunk_num, self.0.node_count()));
                chunk_num += 1;
            }
        }

        Ok(chunks)
    }
}

/// Определяет, является ли нода заголовком без смысловой нагрузки.
///
/// Критерии (достаточно одного):
/// 1. `content_type` — один из структурных заголовков («статья», «раздел», «глава»).
/// 2. Содержимое начинается с известного ключевого слова и короче 300 символов
///    (т.е. нода — это только название, а не норма).
fn is_header_node<C: ToString + Debug + AsRef<str>>(node: &DocumentNode<C>) -> bool
{
    // Типы, которые всегда являются заголовками структурных элементов
    let ct = node.content_type().to_lowercase();
    if matches!(ct.as_str(), "статья" | "раздел" | "глава" | "подраздел")
    {
        return true;
    }

    // Дополнительно: контент начинается с заголовочного слова и короткий
    let content = node.converted_content().as_ref().trim();
    let header_prefixes = ["Статья ", "Раздел ", "Глава ", "Подраздел ", "Параграф "];
    if content.len() < 300 && header_prefixes.iter().any(|p| content.starts_with(p))
    {
        return true;
    }

    false
}

/// Ищет среди предков (от ближайшего к корню) первого, чей текст
/// оканчивается на «:» — признак вводной фразы перечисления.
///
/// `parents` — вектор от корня к непосредственному родителю (порядок из
/// `find_all_parents_by_node`). Итерируем с конца чтобы начать с ближайшего.
fn context_prefix_from_parents<C>(parents: &[&DocumentNode<C>]) -> Option<String>
where
    C: ToString + Debug + AsRef<str>,
{
    for parent in parents.iter().rev()
    {
        let text = parent.converted_content().as_ref().trim();
        if text.ends_with(':')
        {
            return Some(text.to_owned());
        }
    }
    None
}

/// Разбивает текст ноды на чанки.
///
/// Алгоритм:
/// 1. Токенизируем один раз → если помещается, возвращаем как есть.
/// 2. Иначе режем декодированный текст по границам предложений (`find_sentence_boundary`),
///    каждый раз re-токенизируя только оставленную часть (их обычно 2–3).
/// 3. Перекрытие через overlap_tokens: следующий чанк начинается на `overlap` токенов
///    раньше конца предыдущего.
pub(crate) fn split_node_text<E: Encoder>(text: &str, encoder: &E) -> anyhow::Result<Vec<(String, usize)>>
{
    let all_ids: Vec<u32> = encoder.encode(text, false)?.get_ids().to_vec();
    let max = encoder.max_tokens();
    let overlap = encoder.overlap_tokens();

    // Быстрый путь: помещается целиком
    if all_ids.len() <= max
    {
        return Ok(vec![(text.to_owned(), all_ids.len())]);
    }

    let mut chunks: Vec<(String, usize)> = Vec::new();
    let mut start = 0usize; // позиция в all_ids

    while start < all_ids.len()
    {
        let end = (start + max).min(all_ids.len());

        // Декодируем окно токенов в текст
        let window = encoder.decode(&all_ids[start..end], false)?;

        // Последний чанк — берём всё что осталось
        if end >= all_ids.len()
        {
            let trimmed = window.trim().to_owned();
            let token_count = end - start;
            chunks.push((trimmed, token_count));
            break;
        }

        // Ищем границу предложения в декодированном тексте окна
        let boundary_char = find_sentence_boundary(&window);
        let kept = window[..boundary_char].trim().to_owned();

        if kept.is_empty()
        {
            warn!("split_node_text: boundary at start, forcing full window");
            let trimmed = window.trim().to_owned();
            chunks.push((trimmed, end - start));
            start = if end > overlap { end - overlap } else { end };
            continue;
        }

        // Токенизируем только оставленную часть — нужен точный счёт токенов
        let kept_token_count = encoder.encode(&kept, false)?.get_ids().len();
        chunks.push((kept.clone(), kept_token_count));

        // Следующий чанк начинается с перекрытием относительно конца этого
        let next_start = start + kept_token_count;
        start = if next_start > overlap { next_start - overlap } else { 0 };

        if start >= all_ids.len()
        {
            break;
        }
    }

    if chunks.is_empty()
    {
        chunks.push((text.to_owned(), all_ids.len()));
    }

    Ok(chunks)
}

/// Находит позицию символа (byte index) для разрезания в последней четверти текста.
///
/// Приоритет границ (от лучшей к худшей):
///   1. `. `  — конец предложения
///   2. `; `  — конец элемента перечисления
///   3. `, `  — конец клаузы
///   4. `\n`  — конец строки
///   5. ` `   — любой пробел (крайний случай)
///
/// Возвращает длину текста, если ни одна граница не найдена.
fn find_sentence_boundary(text: &str) -> usize
{
    let search_from = text.len() * 3 / 4;
    let tail = &text[search_from..];

    for (needle, offset) in [
        (". ",  1usize),
        ("; ",  1),
        (", ",  1),
    ]
    {
        if let Some(pos) = tail.rfind(needle)
        {
            return search_from + pos + offset;
        }
    }

    if let Some(pos) = tail.rfind('\n')
    {
        return search_from + pos;
    }

    if let Some(pos) = tail.rfind(' ')
    {
        return search_from + pos;
    }

    text.len()
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;
    use tracing::debug;
    use utilites::Date;
    use embedding::{EmbeddingConfiguration, RetriverModel};
    use crate::{chunker::Chunker, systema_actual_converter::SystemaActualConverter};
    use super::SimpleChunker;

    #[tokio::test]
    async fn test_simple_chunker()
    {
        rag_core::init();
        let converter = SystemaActualConverter {};
        let result = systema_client::SystemaClient::get_document(
            Date::new_date(07, 06, 2025),
            "127-ФЗ",
            converter,
        )
        .await
        .unwrap();

        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let model = RetriverModel::new(emb_cfg).await.unwrap();
        let chunker = SimpleChunker::new(result);
        let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
        let chunks = chunker.get_chunks(&model, sender).await.unwrap();
        debug!("SimpleChunker создал {} чанков", chunks.len());
        for chunk in &chunks
        {
            debug!("  path={}\n  content={}\n", chunk.path, chunk.content);
        }
    }
}

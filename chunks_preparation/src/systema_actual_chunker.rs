use rag_core::{Chunk, ChunkMeta, Encoder, ServiceStatus};
use systema_client::{DocumentTree, TreeNode};
use tokio::sync::mpsc::UnboundedSender;

use crate::{chunker::Chunker, simple_chunker::split_node_text};

/// Чанкер на основе `DocumentTree`.
///
/// Иерархия строится через `Content.id` (вида "a3_c8_j1") — без хрупкого
/// поиска по диапазонам абзацев.  Каждый смысловой узел дерева превращается
/// в один или несколько чанков (при превышении лимита токенов).
pub struct SystemaActualChunker(DocumentTree);

impl SystemaActualChunker
{
    pub fn new(tree: DocumentTree) -> Self
    {
        Self(tree)
    }
}

impl Chunker for SystemaActualChunker
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

        for node in self.0.iter()
        {
            let content = node.text.trim();

            // Пустые узлы пропускаем
            if content.is_empty()
            {
                continue;
            }

            // Заголовочные узлы без смысловой нагрузки пропускаем
            if is_header_node(node)
            {
                continue;
            }

            let path = self.0.path_str(node);
            let ancestors = self.0.ancestors(node);
            let context_prefix = context_prefix_from_ancestors(&ancestors);

            let sub_chunks = split_node_text(content, encoder)?;

            for (idx, (text, token_count)) in sub_chunks.into_iter().enumerate()
            {
                //ноду с определением пропускаем, она будет включена в контекст всех детей
                if text.ends_with(":")
                {
                    continue;
                }
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
                    links_hashes: node.links.clone(),
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

/// Заголовочный узел — структурный заголовок без смысловой нагрузки.
fn is_header_node(node: &TreeNode) -> bool
{
    let ct = node.content_type.to_lowercase();
    if matches!(ct.as_str(), "статья" | "раздел" | "глава" | "подраздел")
    {
        return true;
    }
    let content = node.text.trim();
    let header_prefixes = ["Статья ", "Раздел ", "Глава ", "Подраздел ", "Параграф "];
    content.len() < 300 && header_prefixes.iter().any(|p| content.starts_with(p))
}

/// Если ближайший предок заканчивается на «:» — возвращаем его текст
/// как контекстный префикс, чтобы не терять контекст перечисления.
fn context_prefix_from_ancestors(ancestors: &[&TreeNode]) -> Option<String>
{
    for ancestor in ancestors.iter().rev()
    {
        let text = ancestor.text.trim();
        if text.ends_with(':')
        {
            return Some(text.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;
    use tracing::debug;
    use utilites::Date;
    use embedding::{RetriverModel};
    use rag_core::{Converter, EmbeddingConfiguration};
    use crate::{chunker::Chunker, systema_actual_converter::SystemaActualConverter};
    use super::SystemaActualChunker;

    #[tokio::test]
    async fn test_tree_chunker()
    {
        rag_core::init();
        let converter = SystemaActualConverter {};
        let result = systema_client::SystemaClient::get_document_tree(
            Date::new_date(07, 06, 2025),
            "127-ФЗ",
            converter,
        )
        .await
        .unwrap();

        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let model = RetriverModel::new(emb_cfg).await.unwrap();
        let chunker = SystemaActualChunker::new(result);
        let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
        let chunks = chunker.get_chunks(&model, sender).await.unwrap();
        debug!("TreeChunker создал {} чанков", chunks.len());
        for chunk in &chunks
        {
            debug!("  path={}\n  content={}\n", chunk.path, chunk.content);
        }
    }
}

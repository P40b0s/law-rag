use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use scraper::Node;
use systema_client::Converter;
use tracing::{error, info, warn};
use utilites::Date;
use crate::{EmbeddingConfiguration, error::Error, model::Model, retriver::RetriverModel};
use anyhow::{Result, anyhow};

pub struct Chunker<'m>
{
    model: &'m RetriverModel,
}
pub struct ChunkedText
{
    pub content: String,
    pub token_count: usize,
    pub chunk_index: usize,
    pub total_chunks: usize,
}
impl<'m> Chunker<'m>
{
    pub async fn new(model: &'m RetriverModel) -> Result<Self>
    {
        Ok(Self
        {
           model
        })
    }
    
    pub async fn split_text(&self, text: &str) -> Result<Vec<ChunkedText>> {
        // Токенизируем весь документ
        let encoding = self.model.tokenizer().encode(text, false)
            .map_err(|e| anyhow!("Error {} when encode text {}", e, text))?;
        let tokens: Vec<u32> = encoding.get_ids().to_vec();

        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        info!("Splitting text: {} tokens total {}", text, tokens.len());

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;
        let total_tokens = tokens.len();
        let total_chunks = (total_tokens + self.model.max_tokens - 1) / self.model.max_tokens;

        // ВАЖНО: защита от бесконечного цикла
        let max_chunks = 5000; // Максимальное количество чанков
        let mut prev_start = 0;

        while start < total_tokens && chunk_index < max_chunks {
            // Проверка прогресса: убеждаемся, что start увеличивается
            if chunk_index > 0 && start <= prev_start {
                error!("No progress in chunking: start={}, prev_start={}", start, prev_start);
                return Err(anyhow!("Chunking stuck at position {}", start));
            }
            prev_start = start;

            let mut end = std::cmp::min(start + self.model.max_tokens, total_tokens);

            info!("Processing chunk {}: tokens {}..{}", chunk_index, start, end);

            // Если это не последний чанк, ищем оптимальную границу
            if end < total_tokens {
                // Ищем границу для разделения
                if let Some(optimal_end) = self.find_optimal_cutoff(&tokens, start, end, self.model.tokenizer()).await? {
                    end = optimal_end;
                    info!("Found optimal cutoff at {}", end);
                }
            }

            // Проверяем, что end > start
            if end <= start {
                warn!("end <= start, forcing end = start + 1");
                end = start + 1;
            }

            // Декодируем чанк
            let chunk_tokens = &tokens[start..end];
            let chunk_text = self.model.tokenizer().decode(chunk_tokens, false)
                .map_err(|e| anyhow!("Error {} when decode text", e))?;

            let chunk = ChunkedText {
                content: chunk_text,
                token_count: end - start,
                chunk_index,
                total_chunks
            };
            chunks.push(chunk);
            chunk_index += 1;

            // Если достигли конца текста
            if end >= total_tokens {
                info!("Reached end of text at chunk {}", chunk_index);
                break;
            }

            // Перекрытие для следующего чанка
            let next_start = if end > self.model.overlap_tokens {
                end - self.model.overlap_tokens
            } else {
                end
            };

            // ВАЖНО: убедимся, что делаем прогресс
            if next_start <= start {
                warn!("No forward progress: next_start={}, start={}, forcing next_start = start + 1", next_start, start);
                start = start + 1;
            } else {
                start = next_start;
            }

            // Защита от бесконечного цикла
            if start >= total_tokens {
                info!("Start >= total_tokens, breaking");
                break;
            }
        }

        if chunk_index >= max_chunks {
            error!("Reached maximum number of chunks ({}), possible infinite loop", max_chunks);
            return Err(anyhow!("Too many chunks created: {}", chunk_index));
        }

        info!("Successfully created {} chunks", chunks.len());
        Ok(chunks)
    }
//     pub async fn split_text(&self, text: &str) -> Result<Vec<ChunkedText>> {
//     // Токенизируем весь документ
//     let encoding = self.model.tokenizer().encode(text, false)?;
//     let tokens: Vec<u32> = encoding.get_ids().to_vec();
    
//     if tokens.is_empty() {
//         return Ok(Vec::new());
//     }
    
//     info!("Total tokens: {}", tokens.len());
//     info!("Max tokens per chunk: {}", self.model.max_tokens);
//     info!("Overlap tokens: {}", self.model.overlap_tokens);
    
//     let mut chunks = Vec::new();
//     let mut start = 0;
//     let mut chunk_index = 0;
//     let total_tokens = tokens.len();
    
//     // Рассчитываем общее количество чанков для информации
//     let total_chunks = if total_tokens <= self.model.max_tokens {
//         1
//     } else {
//         // Более точный расчет с учетом перекрытия
//         let mut estimated_chunks = 1;
//         let mut pos = self.model.max_tokens;
        
//         while pos < total_tokens {
//             estimated_chunks += 1;
//             pos += self.model.max_tokens - self.model.overlap_tokens;
//         }
//         estimated_chunks
//     };
    
//     while start < total_tokens {
//         // Определяем конец текущего чанка
//         let mut end = std::cmp::min(start + self.model.max_tokens, total_tokens);
        
//         // Если это не последний чанк и есть куда двигаться
//         if end < total_tokens {
//             // Ищем оптимальную границу для разделения
//             match self.find_optimal_cutoff(&tokens, start, end, self.model.tokenizer()).await? {
//                 Some(optimal_end) if optimal_end > start => {
//                     end = optimal_end;
//                 }
//                 _ => {
//                     // Если не нашли хорошую границу, гарантируем минимальный размер чанка
//                     let min_chunk_size = self.model.max_tokens / 2;
//                     if end - start < min_chunk_size && end < total_tokens {
//                         // Расширяем до минимального размера
//                         end = std::cmp::min(start + min_chunk_size, total_tokens);
//                     }
//                 }
//             }
//         }
        
//         // Декодируем чанк
//         let chunk_tokens = &tokens[start..end];
//         let chunk_text = self.model.tokenizer().decode(chunk_tokens, false)?;
        
//         let chunk = ChunkedText {
//             content: chunk_text,
//             token_count: end - start,
//             chunk_index,
//             total_chunks
//         };
        
//         info!("Created chunk {}: tokens [{}, {}), count: {}", 
//             chunk_index, start, end, end - start);
        
//         chunks.push(chunk);
//         chunk_index += 1;
        
//         // Если достигли конца текста
//         if end >= total_tokens {
//             break;
//         }
        
//         // Определяем старт следующего чанка с ПЕРЕКРЫТИЕМ
//         // Мы должны отступить назад от конца текущего чанка на overlap_tokens
//         // но гарантировать, что новый start не будет равен или меньше текущего start
//         let mut next_start = if end > self.model.overlap_tokens {
//             // Отступаем на overlap_tokens от конца
//             end - self.model.overlap_tokens
//         } else {
//             // Если overlap_tokens больше чем end, начинаем с 0
//             0
//         };
        
//         // Важная проверка: следующий start должен быть больше текущего start
//         // Иначе мы можем застрять в бесконечном цикле
//         if next_start <= start {
//             next_start = start + 1;
//         }
        
//         // Проверяем, не вышли ли за пределы
//         if next_start >= total_tokens {
//             break;
//         }
        
//         // Также проверяем, что мы делаем прогресс
//         // Если новый start слишком близко к старому, увеличиваем его
//         if next_start - start < (self.model.max_tokens - self.model.overlap_tokens) / 4 {
//             next_start = start + (self.model.max_tokens - self.model.overlap_tokens) / 2;
            
//             // Но не выходим за пределы
//             if next_start >= total_tokens {
//                 break;
//             }
//         }
        
//         start = next_start;
        
//         // Защита от бесконечного цикла
//         if chunk_index > 1000 { // Максимум 1000 чанков
//             error!("Too many chunks created, possible infinite loop");
//             break;
//         }
//     }
    
//     // Обновляем total_chunks для всех чанков
//     let actual_total_chunks = chunks.len();
//     for chunk in chunks.iter_mut() {
//         chunk.total_chunks = actual_total_chunks;
//     }
    
//     info!("Created {} chunks total", actual_total_chunks);
    
//     // Валидация: проверяем перекрытие между чанками
//     if chunks.len() > 1 {
//         for i in 1..chunks.len() {
//             let prev_end = chunks[i-1].content.len();
//             let curr_start_overlap = if i > 0 {
//                 // Примерная проверка: последние 50 символов предыдущего чанка
//                 // должны встречаться в начале текущего
//                 let overlap_check_len = std::cmp::min(50, prev_end);
//                 let prev_end_text = &chunks[i-1].content[prev_end - overlap_check_len..];
//                 let curr_start_text = &chunks[i].content[..std::cmp::min(overlap_check_len, chunks[i].content.len())];
                
//                 // Простая проверка на перекрытие
//                 let has_overlap = curr_start_text.contains(prev_end_text) || 
//                                  prev_end_text.contains(curr_start_text);
                
//                 if !has_overlap {
//                     warn!("Possible missing overlap between chunk {} and {}", i-1, i);
//                 }
//                 has_overlap
//             } else {
//                 true
//             };
            
//             if !curr_start_overlap {
//                 info!("Chunk {}: '{}...'", i-1, &chunks[i-1].content[..std::cmp::min(30, chunks[i-1].content.len())]);
//                 info!("Chunk {}: '{}...'", i, &chunks[i].content[..std::cmp::min(30, chunks[i].content.len())]);
//             }
//         }
//     }
    
//     Ok(chunks)
// }

    async fn find_optimal_cutoff(
    &self,
    tokens: &[u32],
    start: usize,
    end: usize,
    tokenizer: &Tokenizer,
 ) -> Result<Option<usize>> {
    // Если слишком мало токенов, не ищем границу
    if end - start < 10 {
        return Ok(None);
    }

    // Декодируем весь диапазон для анализа ОДИН РАЗ
    let text_fragment = tokenizer.decode(&tokens[start..end], false)
         .map_err(|e| anyhow!("Error {} when decode text", e))?;

    info!("Finding optimal cutoff for range {}..{} ({} tokens)", start, end, end - start);

    // Ищем границы в обратном порядке (от конца к началу)
    let mut best_boundary: Option<(usize, usize)> = None; // (position, priority)

    // Определяем паттерны границ с приоритетами
    let boundary_patterns = [
        (vec!["\n\n", "\r\n\r\n"], 4),     // Двойной перенос строки - самый высокий приоритет
        (vec![".\n\n", "!\n\n", "?\n\n"], 4), // Конец предложения с двойным переносом
        (vec![".\n", "!\n", "?\n"], 3),    // Конец предложения с переносом
        (vec![". ", "! ", "? "], 3),       // Конец предложения с пробелом
        (vec![".\"", "!\"", "?\""], 3),    // Конец предложения в кавычках
        (vec![".)", "!]","?}"], 3),        // Конец предложения в скобках
        (vec![",\n", ";\n", ":\n"], 2),    // Запятая/точка с запятой с переносом
        (vec![", ", "; ", ": "], 2),       // Запятая/точка с запятой с пробелом
        (vec!["\n", "\r\n"], 1),           // Одиночный перенос строки
        (vec![" ", "\t"], 0),              // Пробел или табуляция
    ];

    // Ищем от 75% до конца диапазона
    let search_start = start + (self.model.max_tokens * 3 / 4);  // 75%
    let search_end = std::cmp::min(end, tokens.len());

    // ВАЖНО: ограничиваем количество итераций для предотвращения зависания
    let max_iterations = 500;  // Максимум 500 проверок
    let mut iterations = 0;

    // Шаг поиска: проверяем каждый 5-й токен для ускорения
    let step = 5;

    let mut i = search_end.saturating_sub(1);
    while i >= search_start && i > start {
        iterations += 1;
        if iterations > max_iterations {
            warn!("Reached max iterations ({}) in find_optimal_cutoff, breaking", max_iterations);
            break;
        }

        if i < tokens.len() {
            let position_in_fragment = i - start;

            // Проверяем все паттерны границ
            for (patterns, priority) in boundary_patterns.iter() {
                for pattern in patterns {
                    // Проверяем, заканчивается ли текст на эту границу
                    if position_in_fragment >= pattern.len() &&
                       text_fragment[..position_in_fragment].ends_with(pattern)
                    {
                        // Быстрая валидация без декодирования (только для точек и запятых)
                        let is_valid = if *pattern == "." || *pattern == "," {
                            self.quick_validate_boundary(&text_fragment, position_in_fragment, pattern)
                        } else {
                            true // Другие паттерны считаем валидными
                        };

                        if is_valid {
                            // Обновляем лучшую границу, если нашли лучше
                            match best_boundary {
                                Some((_, best_priority)) if *priority > best_priority => {
                                    best_boundary = Some((i, *priority));
                                }
                                None => {
                                    best_boundary = Some((i, *priority));
                                }
                                _ => {}
                            }

                            // Если нашли границу высшего приоритета, сразу возвращаем
                            if *priority >= 3 {
                                info!("Found high-priority boundary at position {}", i);
                                return Ok(Some(i));
                            }
                        }
                    }
                }
            }
        }

        // Двигаемся с шагом для ускорения
        if i < step {
            break;
        }
        i -= step;
    }

    // Возвращаем лучшую найденную границу
    if let Some((position, priority)) = best_boundary {
        info!("Found boundary at position {} with priority {}", position, priority);
        // Убедимся, что граница разумная (не слишком близко к началу)
        let min_position = start + self.model.max_tokens / 3;
        if position >= min_position {
            return Ok(Some(position));
        }
    }

    // Если не нашли хорошую границу, попробуем найти границу по словам
    if let Some(word_boundary) = self.find_word_boundary_fast(&text_fragment, start).await? {
        let position = word_boundary;
        if position > start + self.model.max_tokens / 2 {
            info!("Found word boundary at position {}", position);
            return Ok(Some(position));
        }
    }

    warn!("No optimal boundary found in range {}..{}", start, end);
    Ok(None)
}

    /// Быстрая валидация границы без декодирования токенов
    fn quick_validate_boundary(&self, text: &str, position: usize, pattern: &str) -> bool {
        // Проверяем, что это не часть числа (например, "1.23" или "1,000")
        if pattern == "." || pattern == "," {
            // Проверяем символы вокруг точки/запятой в уже декодированном тексте
            if position > 0 && position < text.len() {
                let before = text.chars().nth(position - 1);
                let after = text.chars().nth(position);

                // Если с обеих сторон цифры - это вероятно десятичная дробь
                if before.map(|c| c.is_ascii_digit()).unwrap_or(false) &&
                   after.map(|c| c.is_ascii_digit()).unwrap_or(false)
                {
                    return false;
                }
            }
        }

        // Проверяем аббревиатуры
        let common_abbreviations = [
            "т.д.", "т.п.", "т.е.", "т.к.", "т.н.", "т.о.",
            "др.", "проф.", "акад.", "стр.", "рис.", "гл.",
            "e.g.", "i.e.", "etc.", "vs.", "Mr.", "Mrs.",
            "Dr.", "Prof.", "Inc.", "Ltd.", "Corp.", "Co.",
        ];

        // Проверяем окружение позиции в тексте
        if pattern == "." && position > 10 {
            let start_check = position.saturating_sub(10);
            let end_check = std::cmp::min(position + 3, text.len());
            let context = &text[start_check..end_check];

            for abbr in common_abbreviations.iter() {
                if context.contains(abbr) {
                    return false;
                }
            }

            // Проверяем инициалы (например, "А." или "А.С.")
            if position > 1 {
                let before_chars: String = text.chars().skip(position.saturating_sub(2)).take(2).collect();
                if before_chars.len() == 2 {
                    let chars: Vec<char> = before_chars.chars().collect();
                    if chars[0].is_uppercase() && !chars[0].is_ascii_digit() && chars[1] == '.' {
                        return false;
                    }
                }
            }
        }

        true
    }

async fn is_valid_boundary(
    &self,
    context: &str,
    pattern: &str,
    position: usize,
    tokens: &[u32],
    tokenizer: &Tokenizer,
) -> Result<bool> {
    // Проверяем, что это не часть числа (например, "1.23" или "1,000")
    if pattern == "." || pattern == "," {

        // Проверяем символы вокруг точки/запятой
        let check_window = 3;
        let start_idx = position.saturating_sub(check_window);
        let end_idx = std::cmp::min(position + check_window, tokens.len());

        if start_idx < end_idx {
            let window_text = tokenizer.decode(&tokens[start_idx..end_idx], false)
                .map_err(|e| anyhow!("Error {} when decode text", e))?;

            // Проверяем, окружена ли точка/запятая цифрами
            let pattern_pos = window_text.find(pattern).unwrap_or(0);
            if pattern_pos > 0 && pattern_pos < window_text.len() - 1 {
                let before = window_text.chars().nth(pattern_pos - 1);
                let after = window_text.chars().nth(pattern_pos + 1);

                // Если с обеих стороны цифры - это вероятно десятичная дробь или разделитель тысяч
                if before.map(|c| c.is_ascii_digit()).unwrap_or(false) &&
                   after.map(|c| c.is_ascii_digit()).unwrap_or(false)
                {
                    // Дополнительная проверка: если это запятая и после нее 3 цифры - это разделитель тысяч
                    if pattern == "," && end_idx - position > 4 {
                        let after_text = tokenizer.decode(&tokens[position..std::cmp::min(position + 4, tokens.len())], false)
                            .map_err(|e| anyhow!("Error {} when decode text", e))?;
                        if after_text.chars().take(3).all(|c| c.is_ascii_digit()) {
                            return Ok(false);
                        }
                    }
                    return Ok(false);
                }
            }
        }
    }

    // Проверяем аббревиатуры
    let common_abbreviations = [
        "т.д.", "т.п.", "т.е.", "т.к.", "т.н.", "т.о.",
        "др.", "проф.", "акад.", "стр.", "рис.", "гл.",
        "e.g.", "i.e.", "etc.", "vs.", "Mr.", "Mrs.",
        "Dr.", "Prof.", "Inc.", "Ltd.", "Corp.", "Co.",
        "стр.", "рис.", "табл.", "разд.", "п.", "с.",
    ];

    for abbr in common_abbreviations.iter() {
        if context.contains(abbr) {
            // Проверяем, точно ли это наша граница
            if context.ends_with(abbr) || context.contains(&format!("{}{}", abbr, pattern)) {
                return Ok(false);
            }
        }
    }

    // Проверяем инициалы (например, "А.С. Пушкин")
    if pattern == "." {
        let before_pattern = if context.len() > pattern.len() {
            &context[..context.len() - pattern.len()]
        } else {
            context
        };

        if before_pattern.len() == 1 && before_pattern.chars().all(|c| c.is_uppercase()) {
            return Ok(false);
        }
    }

    Ok(true)
}

    /// Быстрый поиск границы по словам без декодирования токенов
    async fn find_word_boundary_fast(
        &self,
        text: &str,
        start: usize,
    ) -> Result<Option<usize>> {
        // Ищем последний пробел в последней четверти текста
        let search_start = (text.len() * 3 / 4).max(start);

        // Ищем последний пробел, перенос строки или другой разделитель
        for (idx, c) in text[search_start..].char_indices().rev() {
            if c.is_whitespace() {
                let position = start + search_start + idx;
                return Ok(Some(position));
            }
        }

        Ok(None)
    }

async fn find_word_boundary(
    &self,
    tokens: &[u32],
    tokenizer: &Tokenizer,
) -> Result<Option<usize>> {
    // Ищем последний пробел в последней четверти токенов
    let search_start = tokens.len() * 3 / 4;

    for i in (search_start..tokens.len()).rev() {
        if i > 0 {
            let context = tokenizer.decode(&tokens[i-1..std::cmp::min(i+1, tokens.len())], false)
                .map_err(|e| anyhow!("Error {} when decode text", e))?;
            if context.contains(' ') {
                return Ok(Some(i));
            }
        }
    }

    Ok(None)
}

}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk
{
    pub publication_url: String,
    pub document_url: String,
    pub title: String,
    pub number: String,
    pub sign_date: Date,
    pub hash: String,
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub links_hashes: Option<Vec<String>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub embeddings: Option<Vec<f32>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub meta: Option<ChunkMeta>
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ChunkMeta
{
    //http://actual.pravo.gov.ru/list.html#hash=3582241193d46b766ef7d5ae7f8d50577be6bf8a5b6e0d77784769fe1e9628b4
    pub chunk_index: usize,
    pub token_count: usize
}

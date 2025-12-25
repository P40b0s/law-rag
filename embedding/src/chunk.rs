use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use scraper::Node;
use systema_client::Converter;
use tracing::{error, info, warn};
use utilites::Date;
use crate::{error::{Error, Result}, context_model::ContextModel};

pub struct Chunker
{
    model: ContextModel,
}
pub struct ChunkedText
{
    pub content: String,
    pub token_count: usize,
    pub chunk_index: usize,
    pub total_chunks: usize,
}
impl Chunker
{
    pub async fn new() -> Result<Self>
    {
        let model = ContextModel::new(crate::context_model::ModelName::M3).await?;
        Ok(Self
        {
           model
        })
    }
    
    pub async fn split_text(&self, text: &str) -> Result<Vec<ChunkedText>> {
        // Токенизируем весь документ
        let encoding = self.model.tokenizer().encode(text, false)?;
        let tokens: Vec<u32> = encoding.get_ids().to_vec();
        
        if tokens.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;
        let total_tokens = tokens.len();
        let total_chunks = (total_tokens + self.model.max_tokens - 1) / self.model.max_tokens;
        
        while start < total_tokens {
            let mut end = std::cmp::min(start + self.model.max_tokens, total_tokens);
            
            // Если это не последний чанк, ищем оптимальную границу
            if end < total_tokens {
                // Ищем границу для разделения
                if let Some(optimal_end) = self.find_optimal_cutoff(&tokens, start, end, self.model.tokenizer()).await? {
                    end = optimal_end;
                }
            }
            
            // Проверяем, что end > start
            if end <= start {
                end = start + 1;
            }
            
            // Декодируем чанк
            let chunk_tokens = &tokens[start..end];
            let chunk_text = self.model.tokenizer().decode(chunk_tokens, false)?;
            
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
                break;
            }
            
            // Перекрытие для следующего чанка
            start = if end > self.model.overlap_tokens {
                end - self.model.overlap_tokens
            } else {
                end
            };
            
            // Защита от бесконечного цикла
            if start >= total_tokens {
                break;
            }
        }
        
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
    
    // Декодируем весь диапазон для анализа
    let text_fragment = tokenizer.decode(&tokens[start..end], false)?;
    
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
    
    // Ищем от 80% до 95% от максимальной длины
    let search_start = start + (self.model.max_tokens * 4 / 5);  // 80%
    let search_end = std::cmp::min(end, tokens.len());
    
    for i in (search_start..search_end).rev() {
        // Проверяем только если это внутри диапазона чанка
        if i > start && i < tokens.len() {
            let position_in_fragment = i - start;
            
            // Проверяем все паттерны границ
            for (pattern_idx, (patterns, priority)) in boundary_patterns.iter().enumerate() {
                for pattern in patterns {
                    // Проверяем, заканчивается ли текст на эту границу
                    if position_in_fragment >= pattern.len() && 
                       text_fragment[..position_in_fragment].ends_with(pattern) 
                    {
                        // Декодируем контекст для проверки
                        let context_start = if i > 5 { i - 5 } else { 0 };
                        let context_end = std::cmp::min(i + 5, tokens.len());
                        let context = tokenizer.decode(&tokens[context_start..context_end], false)?;
                        
                        // Убеждаемся, что это валидная граница
                        if self.is_valid_boundary(&context, pattern, i, tokens, tokenizer).await? {
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
                                return Ok(Some(i));
                            }
                        }
                    }
                }
            }
        }
        
        // Ограничиваем поиск
        if i < search_start.saturating_sub(self.model.max_tokens / 20) {
            break;
        }
    }
    
    // Возвращаем лучшую найденную границу
    if let Some((position, _)) = best_boundary {
        // Убедимся, что граница разумная (не слишком близко к началу)
        let min_position = start + self.model.max_tokens / 3;
        if position >= min_position {
            return Ok(Some(position));
        }
    }
    
    // Если не нашли хорошую границу, попробуем найти границу по словам
    if let Some(word_boundary) = self.find_word_boundary(&tokens[start..end], tokenizer).await? {
        let position = start + word_boundary;
        if position > start + self.model.max_tokens / 2 {
            return Ok(Some(position));
        }
    }
    
    Ok(None)
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
            let window_text = tokenizer.decode(&tokens[start_idx..end_idx], false)?;
            
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
                        let after_text = tokenizer.decode(&tokens[position..std::cmp::min(position + 4, tokens.len())], false)?;
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

async fn find_word_boundary(
    &self,
    tokens: &[u32],
    tokenizer: &Tokenizer,
) -> Result<Option<usize>> {
    // Ищем последний пробел в последней четверти токенов
    let search_start = tokens.len() * 3 / 4;
    
    for i in (search_start..tokens.len()).rev() {
        if i > 0 {
            let context = tokenizer.decode(&tokens[i-1..std::cmp::min(i+1, tokens.len())], false)?;
            if context.contains(' ') {
                return Ok(Some(i));
            }
        }
    }
    
    Ok(None)
}

}
#[derive(Debug, Serialize, Deserialize)]
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
    pub liks_hashes: Option<Vec<String>>,
    pub embeddings: Option<Vec<f32>>,
    pub meta: Option<ChunkMeta>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkMeta
{
    //http://actual.pravo.gov.ru/list.html#hash=3582241193d46b766ef7d5ae7f8d50577be6bf8a5b6e0d77784769fe1e9628b4
    pub chunk_index: usize,
    pub token_count: usize
}

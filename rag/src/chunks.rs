use serde::{Deserialize, Serialize};
use crate::document::{Document, Section};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chunk 
{
    pub document_uri: String,
    pub document_title: Vec<String>,
    pub document_number: String,
    pub document_sign_date: String,
    pub section_article: Option<String>,
    pub content: String,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChunkMetadata 
{
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub section_index: usize,
    pub char_count: usize,
    pub token_count: Option<usize>, // Для BGE-M3
    pub is_overlap: bool,
}

impl std::fmt::Display for Chunk 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
       let title = if self.document_title.is_empty() {
            "Без названия".to_string()
        } else {
            self.document_title.join(" → ")
        };
        
        let article = self.section_article
            .as_ref()
            .map(|a| format!(" ({})", a))
            .unwrap_or_default();
            
        // Безопасно ограничиваем вывод заголовка
        let title_preview: String = title.chars().take(50).collect();
        if title.chars().count() > 50 {
            write!(
                f, "Чанк {}/{}: {}...{} | {} символов", 
                self.metadata.chunk_index + 1,
                self.metadata.total_chunks,
                title_preview,
                article,
                self.metadata.char_count
            )
        } else {
            write!(
                f, "Чанк {}/{}: {}{} | {} символов", 
                self.metadata.chunk_index + 1,
                self.metadata.total_chunks,
                title,
                article,
                self.metadata.char_count
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkingConfig 
{
    pub max_chunk_size: usize, // в символах (приблизительно соответствует токенам)
    pub chunk_overlap: usize, // в символах
    pub split_by_paragraphs: bool,
    pub paragraph_separator: String,
}

impl Default for ChunkingConfig 
{
    fn default() -> Self 
    {
        Self 
        {
            max_chunk_size: 512, // Оптимально для BGE-M3
            chunk_overlap: 50,
            split_by_paragraphs: true,
            paragraph_separator: "\n\n".to_string(),
        }
    }
}



pub struct DocumentChunker;

impl DocumentChunker {
    /// Основной метод для чанкинга документа
    pub fn chunk_document(
        document: &Document,
        config: &ChunkingConfig,
    ) -> Vec<Chunk> {
        let mut all_chunks = Vec::new();
        let mut global_chunk_index = 0;
        
        for (section_idx, section) in document.sections().iter().enumerate() {
            let section_chunks = if config.split_by_paragraphs {
                Self::chunk_section_by_paragraphs(
                    document,
                    section,
                    section_idx,
                    config,
                    global_chunk_index,
                )
            } else {
                Self::chunk_section_with_sliding_window(
                    document,
                    section,
                    section_idx,
                    config,
                    global_chunk_index,
                )
            };
            
            global_chunk_index += section_chunks.len();
            all_chunks.extend(section_chunks);
        }
        
        // Обновляем total_chunks для всех чанков
        let total_chunks = all_chunks.len();
        for chunk in all_chunks.iter_mut() {
            chunk.metadata.total_chunks = total_chunks;
        }
        
        all_chunks
    }
    
    /// Разбивка по абзацам с учетом перекрытия
    fn chunk_section_by_paragraphs(
        document: &Document,
        section: &Section,
        section_index: usize,
        config: &ChunkingConfig,
        start_chunk_index: usize,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        
        // Разделяем контент на абзацы
        let paragraphs: Vec<String> = if config.paragraph_separator.is_empty() {
            vec![section.content.clone()]
        } else {
            section.content
                .split(&config.paragraph_separator)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        
        let mut current_chunk = String::new();
        let mut current_paragraphs = Vec::new();
        let mut chunk_index = start_chunk_index;
        
        for (para_idx, paragraph) in paragraphs.iter().enumerate() {
            // Если добавление нового абзаца превысит лимит
            if !current_chunk.is_empty() && 
               current_chunk.len() + config.paragraph_separator.len() + paragraph.len() > config.max_chunk_size {
                
                // Сохраняем текущий чанк
                if !current_chunk.is_empty() {
                    let chunk = Self::create_chunk(
                        document,
                        section,
                        section_index,
                        &current_chunk,
                        chunk_index,
                        0,
                        false,
                    );
                    chunks.push(chunk);
                    chunk_index += 1;
                }
                
                // Создаем перекрывающийся чанк (если есть предыдущие абзацы)
                if config.chunk_overlap > 0 && !current_paragraphs.is_empty() {
                    let overlap_text = Self::create_overlap_text(
                        &current_paragraphs,
                        config,
                    );
                    if !overlap_text.is_empty() {
                        let overlap_chunk = Self::create_chunk(
                            document,
                            section,
                            section_index,
                            &overlap_text,
                            chunk_index,
                            0,
                            true,
                        );
                        chunks.push(overlap_chunk);
                        chunk_index += 1;
                        
                        // Начинаем новый чанк с перекрытия
                        current_chunk = overlap_text.clone();
                        current_paragraphs = vec![overlap_text];
                    } else {
                        current_chunk = String::new();
                        current_paragraphs.clear();
                    }
                } else {
                    current_chunk = String::new();
                    current_paragraphs.clear();
                }
            }
            
            // Добавляем абзац к текущему чанку
            if !current_chunk.is_empty() {
                current_chunk.push_str(&config.paragraph_separator);
            }
            current_chunk.push_str(paragraph);
            current_paragraphs.push(paragraph.clone());
            
            // Если абзац сам по себе очень большой, разбиваем его
            if paragraph.len() > config.max_chunk_size {
                let sub_chunks = Self::chunk_large_paragraph(
                    document,
                    section,
                    section_index,
                    paragraph,
                    config,
                    chunk_index,
                    para_idx,
                );
                
                if !sub_chunks.is_empty() {
                    // Сохраняем текущий чанк (если что-то есть)
                    if current_chunk.len() > paragraph.len() {
                        let text_before = current_chunk[..current_chunk.len() - paragraph.len()].to_string();
                        if !text_before.is_empty() {
                            let chunk = Self::create_chunk(
                                document,
                                section,
                                section_index,
                                &text_before,
                                chunk_index,
                                0,
                                false,
                            );
                            chunks.push(chunk);
                            chunk_index += 1;
                        }
                    }
                    chunk_index += sub_chunks.len();
                    chunks.extend(sub_chunks);
                    
                    current_chunk = String::new();
                    current_paragraphs.clear();
                }
            }
        }
        
        // Добавляем последний чанк, если что-то осталось
        if !current_chunk.is_empty() {
            let chunk = Self::create_chunk(
                document,
                section,
                section_index,
                &current_chunk,
                chunk_index,
                0,
                false,
            );
            chunks.push(chunk);
        }
        
        chunks
    }
    
    /// Создание перекрывающегося текста из последних абзацев
    fn create_overlap_text(
        paragraphs: &[String],
        config: &ChunkingConfig,
    ) -> String {
        let mut overlap_text = String::new();
        let mut overlap_size = 0;
        
        // Идем с конца, собираем достаточно текста для перекрытия
        for paragraph in paragraphs.iter().rev() {
            if overlap_size + paragraph.len() > config.chunk_overlap && !overlap_text.is_empty() {
                break;
            }
            
            if !overlap_text.is_empty() {
                overlap_text = format!("{}{}{}", paragraph, config.paragraph_separator, overlap_text);
            } else {
                overlap_text = paragraph.clone();
            }
            overlap_size = overlap_text.len();
        }
        
        overlap_text
    }
    
    /// Разбивка очень больших абзацев
    fn chunk_large_paragraph(
        document: &Document,
        section: &Section,
        section_index: usize,
        paragraph: &str,
        config: &ChunkingConfig,
        start_chunk_index: usize,
        paragraph_index: usize,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut chunk_index = start_chunk_index;
        
        // Используем sliding window для больших абзацев
        let mut start = 0;
        while start < paragraph.len() {
            let end = (start + config.max_chunk_size).min(paragraph.len());
            let mut chunk_end = end;
            
            // Пытаемся закончить на границе предложения
            if end < paragraph.len() {
                if let Some(sentence_end) = Self::get_safe_start(paragraph, end).find(|c: char| c == '.' || c == '!' || c == '?') {
                    chunk_end = end + sentence_end + 1;
                }
            }
            
            let chunk_text = Self::get_safe_slice(paragraph, start, chunk_end).trim().to_string();
            if !chunk_text.is_empty() {
                let is_overlap = start > 0;
                let chunk = Self::create_chunk(
                    document,
                    section,
                    section_index,
                    &chunk_text,
                    chunk_index,
                    paragraph_index,
                    is_overlap,
                );
                chunks.push(chunk);
                chunk_index += 1;
            }
            
            // Перемещаемся с перекрытием
            start = if chunk_end - start > config.chunk_overlap {
                chunk_end - config.chunk_overlap
            } else {
                chunk_end
            };
            
            // Защита от бесконечного цикла
            if start >= paragraph.len() || start == chunk_end {
                break;
            }
        }
        
        chunks
    }
    
    /// Альтернативный метод: скользящее окно (без учета абзацев)
    fn chunk_section_with_sliding_window(
        document: &Document,
        section: &Section,
        section_index: usize,
        config: &ChunkingConfig,
        start_chunk_index: usize,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let text = &section.content;
        let mut chunk_index = start_chunk_index;
        let mut start = 0;
        
        while start < text.len() {
            let end = (start + config.max_chunk_size).min(text.len());
            let mut chunk_end = end;
            
            // Пытаемся найти хорошую границу для чанка
            if end < text.len() {
                // Ищем границу предложения
                if let Some(sentence_end) = Self::get_safe_start(text, end).find(|c: char| c == '.' || c == '!' || c == '?') {
                    chunk_end = end + sentence_end + 1;
                }
                // Или границу абзаца
                else if let Some(para_end) = Self::get_safe_start(text, end).find("\n\n") {
                    chunk_end = end + para_end + 2;
                }
            }
            
            let chunk_text = Self::get_safe_slice(text, start, chunk_end).trim().to_string();
            if !chunk_text.is_empty() {
                let is_overlap = start > 0;
                let chunk = Self::create_chunk(
                    document,
                    section,
                    section_index,
                    &chunk_text,
                    chunk_index,
                    0,
                    is_overlap,
                );
                chunks.push(chunk);
                chunk_index += 1;
            }
            
            // Перемещаемся с перекрытием
            start = if chunk_end - start > config.chunk_overlap {
                chunk_end - config.chunk_overlap
            } else {
                chunk_end
            };
        }
        
        chunks
    }
    
    /// Создание объекта чанка
    fn create_chunk(
        document: &Document,
        section: &Section,
        section_index: usize,
        content: &str,
        chunk_index: usize,
        paragraph_index: usize,
        is_overlap: bool,
    ) -> Chunk {
        Chunk {
            document_uri: document.uri().to_string(),
            document_title: document.title().clone(),
            section_article: section.article.clone(),
            document_number: document.number().to_owned(),
            document_sign_date: document.date().to_owned(),
            content: content.to_string(),
            metadata: ChunkMetadata {
                chunk_index,
                total_chunks: 0, // Будет обновлено позже
                section_index,
                char_count: content.chars().count(),
                token_count: None, // Можно вычислить с помощью токенизатора BGE-M3
                is_overlap,
            },
        }
    }
    
    /// Получение текста для эмбеддингов (очищенный текст)
    pub fn get_embedding_text(chunk: &Chunk) -> String {
        // Можно добавить дополнительные метаданные для контекста
        let mut text = String::new();
        
        // Добавляем заголовок документа, если есть
        if !chunk.document_title.is_empty() {
            let title = chunk.document_title.join(" ");
            text.push_str(&format!("Документ: {}\n", title));
        }
        
        // Добавляем статью из секции, если есть
        if let Some(article) = &chunk.section_article {
            text.push_str(&format!("Статья: {}\n", article));
        }

        
        // Добавляем основной контент
        text.push_str(&chunk.content);
        
        text
    }
    /// Безопасное получение превью текста (первые N символов)
    pub fn get_safe_preview(text: &str, max_chars: usize) -> String 
    {
        text.chars()
            .take(max_chars)
            .collect()
    }
    pub fn get_safe_slice(text: &str, start: usize, end: usize) -> String 
    {
        text.chars()
        .skip(start)
            .take(end-start)
            .collect()
    }
    pub fn get_safe_start(text: &str, start: usize) -> String 
    {
        text.chars()
        .skip(start)
        .collect()
    }
    
    /// Безопасный срез строки до указанного количества символов
    pub fn safe_truncate(text: &str, max_chars: usize) -> String 
    {
        if text.chars().count() <= max_chars {
            return text.to_string();
        }
        
        let mut result = String::new();
        let mut count = 0;
        
        for ch in text.chars() {
            if count >= max_chars {
                break;
            }
            result.push(ch);
            count += 1;
        }
        
        result
    }
}


#[cfg(test)]
mod tests
{
    use crate::{chunks::{ChunkingConfig, DocumentChunker}, document::{Document, Section}};
    fn create_test_document() -> Document {
        let mut doc = Document::new("law://fz-273/article-36", "2012-12-12".to_owned(), "273-ФЗ");
        doc.add_title("Федеральный закон 'Об образовании'".to_string());
        
        // Добавляем секции как в вашем примере
        doc.add_section(Section {
            article: Some("Статья 36. Стипендии и другие денежные выплаты".to_string()),
            content: "5. Государственная социальная стипендия назначается студентам, являющимся детьми-сиротами и детьми, оставшимися без попечения родителей, лицами из числа детей-сирот и детей, оставшихся без попечения родителей, лицами, потерявшими в период обучения обоих родителей или единственного родителя, детьми-инвалидами, инвалидами I и II групп, инвалидами с детства, студентами, подвергшимися воздействию радиации вследствие катастрофы на Чернобыльской АЭС и иных радиационных катастроф, вследствие ядерных испытаний на Семипалатинском полигоне, студентами, являющимися инвалидами вследствие военной травмы или заболевания, полученных в период прохождения военной службы, и ветеранами боевых действий, а также студентами из числа граждан, проходивших в течение не менее трех лет военную службу по контракту на воинских должностях, подлежащих замещению солдатами, матросами, сержантами, старшинами, и уволенных с военной службы по основаниям, предусмотренным подпунктами \"б\" - \"г\" пункта 1, подпунктом \"а\" пункта 2 и подпунктами \"а\" - \"в\" пункта 3 статьи 51 Федерального закона от 28 марта 1998 года № 53-ФЗ \"О воинской обязанности и военной службе\". Государственная социальная стипендия назначается также студентам, получившим государственную социальную помощь. Государственная социальная стипендия назначается указанной категории студентов со дня представления в организацию, осуществляющую образовательную деятельность, документа, подтверждающего назначение государственной социальной помощи, на один год со дня назначения указанной государственной социальной помощи. (В редакции Федерального закона от 29.12.2017 № 473-ФЗ)".to_string(),
        });
        
        doc.add_section(Section {
            article: Some("Статья 36. Стипендии и другие денежные выплаты".to_string()),
            content: "6. Аспирантам, ординаторам, ассистентам-стажерам, обучающимся по очной форме обучения за счет бюджетных ассигнований федерального бюджета, в порядке, установленном федеральным органом исполнительной власти, осуществляющим функции по выработке и реализации государственной политики и нормативно-правовому регулированию в сфере высшего образования, назначаются государственные стипендии. (В редакции Федерального закона от 26.07.2019 № 232-ФЗ)".to_string(),
        });
        
        doc
    }
    
    #[test]
    fn test_chunking() {
        let document = create_test_document();
        let config = ChunkingConfig {
            max_chunk_size: 2000,
            chunk_overlap: 100,
            split_by_paragraphs: true,
            paragraph_separator: "\n\n".to_string(),
        };
        
        let chunks = DocumentChunker::chunk_document(&document, &config);
        
        println!("Всего чанков: {}", chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            println!("{}. {}", i + 1, chunk);
            
            // Безопасный вывод первых 100 символов (не байтов!)
            let preview_length = chunk.content.chars().count().min(100);
            let preview: String = chunk.content.chars().take(preview_length).collect();
            println!("Контент: {}...", preview);
            println!("---");
        }
        
        // Получаем текст для эмбеддингов
        if let Some(first_chunk) = chunks.first() {
            let embedding_text = DocumentChunker::get_embedding_text(first_chunk);
            // Безопасный вывод первых 200 символов
            let preview_length = embedding_text.chars().count().min(200);
            let preview: String = embedding_text.chars().take(preview_length).collect();
            println!("Текст для эмбеддинга: {}...", preview);
        }
    }
    
    #[test]
    fn test_chunking_with_large_content() {
        // Документ с очень большим контентом
        let mut doc = Document::new("test://large-doc", "2012-12-12".to_owned(), "273-ФЗ");
        doc.add_title("Тестовый документ".to_string());
        
        // Создаем очень большой текст
        let mut large_content = String::new();
        for i in 0..10 {
            large_content.push_str(&format!("Абзац {}. Это тестовый абзац с некоторым содержанием. ", i + 1));
            large_content.push_str("Он содержит несколько предложений для демонстрации работы чанкера. ");
            large_content.push_str("Здесь может быть любой текст, который нужно разбить на части.\n\n");
        }
        
        doc.add_section(Section {
            article: Some("Большая секция".to_string()),
            content: large_content,
        });
        
        let config = ChunkingConfig::default();
        let chunks = DocumentChunker::chunk_document(&doc, &config);
        
        assert!(!chunks.is_empty(), "Должны быть созданы чанки");
        println!("Создано {} чанков для большого документа", chunks.len());
        
        // Проверяем, что нет пустых чанков
        for chunk in &chunks {
            assert!(!chunk.content.is_empty(), "Чанк не должен быть пустым");
            assert!(chunk.content.chars().count() <= config.max_chunk_size, 
                   "Чанк не должен превышать максимальный размер");
        }
    }
    
    // Добавим вспомогательные функции для безопасной работы со строками
    #[test]
    fn test_safe_string_operations() {
        let text = "Привет, мир! Hello, world!";
        
        // Небезопасно (может упасть на границе символов UTF-8):
        // let slice = &text[0..5]; // Может упасть
        
        // Безопасно:
        let chars_count = text.chars().count();
        println!("Количество символов: {}", chars_count);
        
        // Взять первые N символов:
        let first_5_chars: String = text.chars().take(5).collect();
        println!("Первые 5 символов: {}", first_5_chars);
        
        // Найти границу символа для среза:
        let take_bytes = 10;
        let mut char_boundary = 0;
        for (idx, _) in text.char_indices() {
            if idx >= take_bytes {
                break;
            }
            char_boundary = idx;
        }
        
        // Теперь можно безопасно сделать срез:
        let safe_slice = &text[..char_boundary];
        println!("Безопасный срез: {}", safe_slice);
    }


}
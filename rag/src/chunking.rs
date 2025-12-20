use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use tracing::info;
use crate::error::{Error, Result};
use crate::model::LongContextModel;
use scraper::{ElementRef, Html, Node, Selector};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub metadata: DocumentMetadata,
    // embedding: Vec<f32>, // Убрано - будет формироваться отдельно
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_url: String,
    pub title: String,
    
    // Для чанкинга
    pub chunk_index: usize,
    pub total_chunks: usize,
    
    // Для иерархической структуры
    // pub parent_chunk_id: Option<String>,
    
    // Контентная информация
    pub tags: Vec<String>,
    pub timestamp: Option<String>,
    
    pub token_count: usize,
    
    // Для длинных документов
    pub start_token: usize,
    pub end_token: usize,
}

impl DocumentMetadata {
    pub fn new(source_url: String, title: String) -> Self {
        Self { 
            source_url,
            title,
            chunk_index: 0,
            total_chunks: 0,
            tags: Vec::new(),
            timestamp: None,
            token_count: 0,
            start_token: 0,
            end_token: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentSection {
    pub heading: String,
    pub level: u32,
    pub content: String,
    pub section_type: SectionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionType {
    Title,      // Заголовок документа
    Header,     // Статья закона
    Paragraph,   // Пункт/подпункт
    Amendment,   // Изменение/дополнение
    Signature,   // Подписи/даты
    Citation,    // Ссылки на законы
}

pub struct LegalDocumentChunker {
    pub max_tokens: usize,
    pub overlap_tokens: usize,
    pub model_type: LongContextModel,
    pub preserve_structure: bool,
}

impl LegalDocumentChunker {
    pub fn new(model_type: LongContextModel) -> Self {
        let max_tokens = model_type.max_tokens();
        
        Self {
            max_tokens,
            overlap_tokens: (max_tokens / 10).min(256), // Максимум 256 токенов перекрытия
            model_type,
            preserve_structure: true,
        }
    }
    
    pub async fn chunk_document(
        &self,
        html: Html,
        metadata: &DocumentMetadata,
        tokenizer: &Tokenizer,
    ) -> Result<Vec<DocumentChunk>> {
        if self.preserve_structure {
            self.semantic_chunking(html, metadata, tokenizer).await
        } else {
            self.simple_chunking(html, metadata, tokenizer).await
        }
    }
    
    async fn semantic_chunking(
        &self,
        html: Html,
        metadata: &DocumentMetadata,
        tokenizer: &Tokenizer,
    ) -> Result<Vec<DocumentChunk>> {
        // 1. Извлекаем структуру юридического документа
        let sections = self.extract_legal_structure(html).await?;
        
        // 2. Группируем секции в чанки с сохранением смысловых границ
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_tokens = 0;
        let mut chunk_index = 0;
        let mut section_infos = Vec::new();
        
        for section in sections {
            let section_content = if !section.heading.is_empty() {
                format!("{}\n{}", section.heading, section.content)
            } else {
                section.content.clone()
            };
            
            let section_tokens = tokenizer.encode(section_content.clone(), false)?.len();
            
            // Проверяем, нужно ли начать новый чанк
            if !current_chunk.is_empty() && 
               (current_tokens + section_tokens > self.max_tokens ||
                self.should_start_new_chunk(&section.section_type)) 
            {
                let chunk = self.create_legal_chunk(
                    &current_chunk, 
                    metadata, 
                    chunk_index,
                    &section_infos,
                    tokenizer
                ).await?;
                chunks.push(chunk);
                
                // Сохраняем перекрытие для контекста
                if self.overlap_tokens > 0 && !chunks.is_empty() {
                    self.add_overlap_to_previous_chunk(&mut chunks, tokenizer).await?;
                }
                
                current_chunk = section_content;
                current_tokens = section_tokens;
                section_infos = vec![section.section_type];
                chunk_index += 1;
            } else {
                // Добавляем к текущему чанку
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(&section_content);
                current_tokens += section_tokens;
                section_infos.push(section.section_type);
            }
        }
        
        // Добавляем последний чанк
        if !current_chunk.is_empty() {
            let chunk = self.create_legal_chunk(
                &current_chunk, 
                metadata, 
                chunk_index,
                &section_infos,
                tokenizer
            ).await?;
            chunks.push(chunk);
        }
        
        // Обновляем metadata для всех чанков
        let total_chunks = chunks.len();
        for (i, chunk) in chunks.iter_mut().enumerate() {
            chunk.metadata.chunk_index = i;
            chunk.metadata.total_chunks = total_chunks;
        }
        
        Ok(chunks)
    }
    
    async fn extract_legal_structure(&self, html: Html) -> Result<Vec<DocumentSection>> 
    {
        let body_selector = Selector::parse("body").unwrap();
        let body = html.select(&body_selector).next().unwrap();
        
        let mut sections = Vec::new();
        let mut current_section = DocumentSection {
            heading: String::new(),
            level: 1,
            content: String::new(),
            section_type: SectionType::Header,
        };
        
        for node in body.children() 
        {
            if let Some(element) = node.value().as_element() 
            {
                if let Some(element_ref) = ElementRef::wrap(node) 
                {
                    let tag_name = element.name();
                    let text = element_ref.text().collect::<String>().trim().to_string();
                    info!("Обработка тега: {}, текст: {}", tag_name, text);
                    if text.is_empty() || text == " " || text == "&nbsp;" 
                    {
                        continue;
                    }
                    
                    // Определяем тип секции на основе классов и контента
                    let (section_type, is_new_section) = self.classify_legal_element(element, &text);
                    
                    if is_new_section && !current_section.content.is_empty() 
                    {
                        sections.push(current_section.clone());
                        current_section = DocumentSection 
                        {
                            heading: if section_type == SectionType::Header 
                            {
                                text.clone()
                            } 
                            else 
                            {
                                String::new()
                            },
                            level: match section_type 
                            {
                                SectionType::Header => 1,
                                SectionType::Paragraph => 2,
                                _ => 3,
                            },
                            content: if section_type != SectionType::Header 
                            {
                                text.clone()
                            } 
                            else 
                            {
                                String::new()
                            },
                            section_type,
                        };
                    } 
                    else 
                    {
                        if !current_section.content.is_empty() 
                        {
                            current_section.content.push_str("\n");
                        }
                        current_section.content.push_str(&text);
                    }
                }
            }
        }
        
        // Добавляем последнюю секцию
        if !current_section.content.is_empty() {
            sections.push(current_section);
        }
        
        // Объединяем мелкие секции
        Ok(self.merge_small_sections(sections, 100)) // Объединяем секции меньше 100 символов
    }
    
    fn classify_legal_element(&self, element: &scraper::node::Element, text: &str) -> (SectionType, bool) {
        let classes: Vec<&str> = element.classes().collect();
        
        // Анализируем по классам
        if classes.contains(&"C") 
        {
            return (SectionType::Citation, true);
        } 
        else if classes.contains(&"T") 
        {
            return (SectionType::Title, true);
        } 
        else if classes.contains(&"H") 
        {
            return (SectionType::Header, true);
        } 
        else if classes.contains(&"I") 
        {
            return (SectionType::Signature, true);
        }
        
        // // Анализируем по текстовому содержанию
        // if text.starts_with("Статья") || text.starts_with("СТАТЬЯ") 
        // {
        //     return (SectionType::Article, true);
        // } 
        // else if text.starts_with("ФЕДЕРАЛЬНЫЙ ЗАКОН") || text.contains("РОССИЙСКАЯ ФЕДЕРАЦИЯ") 
        // {
        //     return (SectionType::Header, true);
        // } 
        // else if text.starts_with("Принят") || text.starts_with("Одобрен") || 
        //         text.contains("Президент") || text.contains("Путин") 
        // {
        //     return (SectionType::Signature, true);
        // } 
        // else if text.contains("изложить в следующей редакции") || 
        //         text.contains("дополнить") || text.contains("внести изменения") 
        // {
        //     return (SectionType::Amendment, true);
        // } 
        // else if text.contains("Федеральный закон") && text.contains("№") && text.contains("ФЗ") 
        // {
        //     return (SectionType::Citation, true);
        // }
        
        // По умолчанию - параграф
        (SectionType::Paragraph, false)
    }
    
    fn should_start_new_chunk(&self, section_type: &SectionType) -> bool 
    {
        // Начинаем новый чанк на важных границах
        match section_type 
        {
            SectionType::Header => true,
            SectionType::Paragraph => true,
            _ => false,
        }
    }
    
    async fn create_legal_chunk(
        &self,
        content: &str,
        metadata: &DocumentMetadata,
        chunk_index: usize,
        section_types: &[SectionType],
        tokenizer: &Tokenizer,
    ) -> Result<DocumentChunk> {
        let token_count = tokenizer.encode(content, false)?.len();
        
        let mut enhanced_metadata = metadata.clone();
        enhanced_metadata.chunk_index = chunk_index;
        enhanced_metadata.token_count = token_count;
        
        // Добавляем информацию о типах секций в метаданные
        let section_type_names: Vec<String> = section_types.iter()
            .map(|t| format!("{:?}", t))
            .collect();
        if !section_type_names.is_empty() {
            enhanced_metadata.tags.push("legal_document".to_string());
            enhanced_metadata.tags.extend(section_type_names);
        }
        
        Ok(DocumentChunk {
            id: format!("{}_{}", metadata.source_url, chunk_index),
            content: content.to_string(),
            metadata: enhanced_metadata,
        })
    }
    
    async fn simple_chunking(
        &self,
        html: Html,
        metadata: &DocumentMetadata,
        tokenizer: &Tokenizer,
    ) -> Result<Vec<DocumentChunk>> {
        // Очищаем HTML и получаем чистый текст
        let text = self.extract_text_from_html(html).await?;
        
        // Токенизируем весь документ
        let encoding = tokenizer.encode(text.clone(), false)?;
        let tokens: Vec<u32> = encoding.get_ids().to_vec();
        
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;
        let total_chunks = (tokens.len() + self.max_tokens - 1) / self.max_tokens;
        
        while start < tokens.len() {
            let mut end = std::cmp::min(start + self.max_tokens, tokens.len());
            
            // Пытаемся закончить на границе предложения/абзаца
            if end < tokens.len() {
                end = self.find_optimal_cutoff(&tokens, start, end, tokenizer).await?;
            }
            
            let chunk_tokens = &tokens[start..end];
            let chunk_text = tokenizer.decode(chunk_tokens, false)?;
            
            let chunk = DocumentChunk {
                id: format!("{}_{}", metadata.source_url, chunk_index),
                content: chunk_text,
                metadata: DocumentMetadata {
                    source_url: metadata.source_url.clone(),
                    title: metadata.title.clone(),
                    chunk_index,
                    total_chunks,
                    tags: metadata.tags.clone(),
                    timestamp: metadata.timestamp.clone(),
                    token_count: end - start,
                    start_token: start,
                    end_token: end,
                },
            };
            
            chunks.push(chunk);
            chunk_index += 1;
            
            if end == tokens.len() {
                break;
            }
            
            start = end.saturating_sub(self.overlap_tokens);
        }
        
        Ok(chunks)
    }
    
    async fn extract_text_from_html(&self, html: Html) -> Result<String> {
        let p_selector = Selector::parse("p").unwrap();
        
        let mut text_parts = Vec::new();
        
        for element in html.select(&p_selector) {
            let text = element.text().collect::<String>();
            let trimmed = text.trim();
            
            if !trimmed.is_empty() && trimmed != " " && !trimmed.chars().all(|c| c.is_whitespace()) {
                text_parts.push(trimmed.to_string());
            }
        }
        
        Ok(text_parts.join("\n\n"))
    }
    
    async fn find_optimal_cutoff(
        &self,
        tokens: &[u32],
        start: usize,
        mut end: usize,
        tokenizer: &Tokenizer,
    ) -> Result<usize> {
        // Ищем границы предложений или пунктов для лучшего разделения
        let text = tokenizer.decode(&tokens[start..end], false)?;
        
        // Ищем последнюю значимую границу
        let boundary_chars = ['.', ';', ':', '!', '?', ')', ']', '}'];
        
        for i in (start..end).rev() 
        {
            if i <= start + self.max_tokens / 2 
            {
                break; // Не идем слишком далеко назад
            }
            
            // Декодируем небольшой фрагмент для проверки
            if i > 0 
            {
                let prev_text = tokenizer.decode(&tokens[i-1..=i], false)?;
                if boundary_chars.iter().any(|&c| prev_text.ends_with(c)) 
                {
                    // Проверяем, что это не часть числа или аббревиатуры
                    let context = tokenizer.decode(&tokens[i-3..=i], false)?;
                    if !context.chars().last().map_or(false, |c| c.is_ascii_digit()) &&
                       !context.ends_with("т.д.") && !context.ends_with("т.п.") 
                    {
                        return Ok(i);
                    }
                }
            }
        }
        
        Ok(end)
    }
    
    fn merge_small_sections(&self, sections: Vec<DocumentSection>, min_size: usize) -> Vec<DocumentSection> 
    {
        let mut merged = Vec::new();
        let mut current: Option<DocumentSection> = None;
        
        for section in sections 
        {
            match current.take() 
            {
                Some(mut prev) => 
                {
                    if section.content.len() < min_size && 
                       prev.section_type == section.section_type &&
                       prev.level == section.level 
                    {
                        // Объединяем мелкие секции одного типа
                        prev.content.push_str("\n\n");
                        prev.content.push_str(&section.content);
                        current = Some(prev);
                    } 
                    else 
                    {
                        merged.push(prev);
                        current = Some(section);
                    }
                }
                None => current = Some(section),
            }
        }
        
        if let Some(last) = current {
            merged.push(last);
        }
        
        merged
    }
    
    async fn add_overlap_to_previous_chunk(
    &self,
    chunks: &mut Vec<DocumentChunk>,
    tokenizer: &Tokenizer,
) -> Result<()> {
    if chunks.len() < 2 {
        return Ok(());
    }
    
    let current_idx = chunks.len() - 1;
    let prev_idx = current_idx - 1;
    
    let prev_chunk = &chunks[prev_idx];
    let prev_encoding = tokenizer.encode(prev_chunk.content.clone(), false)?;
    let prev_tokens = prev_encoding.get_ids();
    
    // Берем последние overlap_tokens из предыдущего чанка (без специальных токенов)
    let overlap_start = prev_tokens.len().saturating_sub(self.overlap_tokens);
    let overlap_tokens = &prev_tokens[overlap_start..];
    
    if !overlap_tokens.is_empty() {
        // Декодируем без добавления специальных токенов
        let overlap_text = tokenizer.decode(overlap_tokens, true)?; // true = skip_special_tokens
        
        // Убираем возможные специальные токены, которые могли остаться
        let clean_overlap = overlap_text
            .replace("<s>", "")
            .replace("</s>", "")
            .replace("<pad>", "")
            .trim()
            .to_string();
        
        if !clean_overlap.is_empty() {
            // Добавляем перекрытие к текущему чанку
            let new_content = format!("{}\n\n{}", clean_overlap, chunks[current_idx].content);
            chunks[current_idx].content = new_content;
            
            // Обновляем метаданные
            let new_token_count = tokenizer.encode(chunks[current_idx].content.clone(), false)?.len();
            chunks[current_idx].metadata.token_count = new_token_count;
        }
    }
    
    Ok(())
}
    
    // Дополнительный метод для извлечения структурированной информации
    pub async fn extract_legal_metadata(&self, html: &str) -> Result<HashMap<String, String>> {
        let document = Html::parse_document(html);
        let mut metadata = HashMap::new();
        
        // Извлекаем номер закона
        let number_selector = Selector::parse("p.I").unwrap();
        for element in document.select(&number_selector) {
            let text = element.text().collect::<String>();
            if text.contains("№") && text.contains("ФЗ") {
                metadata.insert("law_number".to_string(), text);
                break;
            }
        }
        
        // Извлекаем дату принятия
        let date_selectors = [
            Selector::parse("p.I").unwrap(),
            Selector::parse("p.T").unwrap(),
        ];
        
        for selector in date_selectors.iter() {
            for element in document.select(selector) {
                let text = element.text().collect::<String>();
                if text.contains("декабря") || text.contains("года") {
                    metadata.insert("date".to_string(), text);
                    break;
                }
            }
            if metadata.contains_key("date") {
                break;
            }
        }
        
        // Извлекаем название
        let title_selector = Selector::parse("p.T").unwrap();
        if let Some(element) = document.select(&title_selector).nth(1) {
            metadata.insert("title".to_string(), element.text().collect());
        }
        
        Ok(metadata)
    }
}
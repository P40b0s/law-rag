// use serde::{Deserialize, Serialize};

// use crate::chunking::DocumentMetadata;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct DocumentSection {
//     pub heading: String,
//     pub level: u8, // 1-6 для h1-h6
//     pub content: String,
//     pub start_position: usize, // Позиция в исходном тексте
//     pub end_position: usize,
//     pub html_tag: Option<String>, // Например: "h1", "h2", "div.section"
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct DocumentStructure {
//     pub title: Option<String>,
//     pub sections: Vec<DocumentSection>,
//     pub metadata: DocumentMetadata,
//     pub outline: Vec<OutlineItem>, // Иерархическая структура
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct OutlineItem {
//     pub id: String,
//     pub title: String,
//     pub level: usize,
//     pub children: Vec<OutlineItem>,
//     pub section_index: Option<usize>, // Ссылка на секцию
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct HierarchicalEmbeddings {
//     pub full_document: Vec<f32>, // Эмбеддинг всего документа (суммированный)
//     pub section_embeddings: Vec<SectionEmbedding>,
//     pub paragraph_embeddings: Vec<ParagraphEmbedding>,
//     pub hierarchical_index: Vec<HierarchicalIndexEntry>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SectionEmbedding {
//     pub section_id: String,
//     pub embedding: Vec<f32>,
//     pub metadata: SectionMetadata,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SectionMetadata {
//     pub heading: String,
//     pub level: u8,
//     pub token_count: usize,
//     pub word_count: usize,
//     pub contains_tables: bool,
//     pub contains_code: bool,
//     pub contains_images: bool,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ParagraphEmbedding {
//     pub paragraph_id: String,
//     pub section_id: String,
//     pub embedding: Vec<f32>,
//     pub text: String,
//     pub position_in_section: usize,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct HierarchicalIndexEntry {
//     pub node_id: String,
//     pub parent_id: Option<String>,
//     pub node_type: NodeType, // "document", "section", "paragraph"
//     pub embedding: Vec<f32>,
//     pub metadata: IndexMetadata,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub enum NodeType {
//     Document,
//     Section,
//     Paragraph,
//     List,
//     Table,
//     CodeBlock,
//     ImageCaption,
// }


// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub enum DocumentType {
//     Article,
//     BlogPost,
//     Documentation,
//     ResearchPaper,
//     BookChapter,
//     LegalDocument,
//     TechnicalSpec,
//     Tutorial,
//     News,
//     ForumPost,
//     CodeDocumentation,
// }



// use scraper::{Html, Selector, ElementRef};
// use kuchiki::{NodeData, NodeRef};

// pub struct DocumentStructureParser {
//     pub preserve_semantic_elements: bool,
//     pub extract_tables: bool,
//     pub extract_code_blocks: bool,
//     pub max_section_length: usize, // в токенах
// }

// impl DocumentStructureParser {
//     pub fn new() -> Self {
//         Self {
//             preserve_semantic_elements: true,
//             extract_tables: true,
//             extract_code_blocks: true,
//             max_section_length: 2000,
//         }
//     }
    
//     pub async fn parse_html(&self, html: &str, url: &str) -> Result<DocumentStructure> {
//         let document = Html::parse_document(html);
        
//         // Извлекаем заголовок
//         let title = self.extract_title(&document);
        
//         // Извлекаем метаданные
//         let metadata = self.extract_metadata(&document, url).await?;
        
//         // Извлекаем иерархическую структуру
//         let sections = self.extract_sections(&document).await?;
        
//         // Строим outline
//         let outline = self.build_outline(&sections);
        
//         Ok(DocumentStructure {
//             title,
//             sections,
//             metadata,
//             outline,
//         })
//     }
    
//     async fn extract_sections(&self, document: &Html) -> Result<Vec<DocumentSection>> {
//         let mut sections = Vec::new();
//         let mut current_position = 0;
        
//         // Находим все структурные элементы
//         let structural_selectors = [
//             ("h1", 1),
//             ("h2", 2),
//             ("h3", 3),
//             ("h4", 4),
//             ("h5", 5),
//             ("h6", 6),
//             ("section", 0),
//             ("article", 0),
//             ("div.chapter", 0),
//             ("div.section", 0),
//         ];
        
//         for (selector_str, default_level) in structural_selectors.iter() {
//             if let Ok(selector) = Selector::parse(selector_str) {
//                 for element in document.select(&selector) {
//                     let section = self.create_section_from_element(
//                         element, 
//                         *default_level, 
//                         &mut current_position
//                     ).await?;
                    
//                     if !section.content.trim().is_empty() {
//                         sections.push(section);
//                     }
//                 }
//             }
//         }
        
//         // Если не нашли структурных элементов, создаем одну большую секцию
//         if sections.is_empty() {
//             let body_text = document
//                 .select(&Selector::parse("body").unwrap())
//                 .next()
//                 .map(|e| e.text().collect::<String>())
//                 .unwrap_or_default();
            
//             if !body_text.trim().is_empty() {
//                 sections.push(DocumentSection {
//                     heading: "Document".to_string(),
//                     level: 1,
//                     content: body_text,
//                     start_position: 0,
//                     end_position: body_text.len(),
//                     html_tag: Some("body".to_string()),
//                 });
//             }
//         }
        
//         Ok(sections)
//     }
    
//     async fn create_section_from_element(
//         &self,
//         element: ElementRef,
//         default_level: u8,
//         current_position: &mut usize,
//     ) -> Result<DocumentSection> {
//         let tag_name = element.value().name();
//         let heading = element.text().collect::<String>().trim().to_string();
        
//         // Определяем уровень заголовка
//         let level = if tag_name.starts_with('h') && tag_name.len() == 2 {
//             tag_name.chars().nth(1).unwrap().to_digit(10).unwrap() as u8
//         } else {
//             default_level
//         };
        
//         // Собираем контент до следующего структурного элемента
//         let content = self.collect_content_until_next_heading(element).await?;
        
//         let section = DocumentSection {
//             heading,
//             level,
//             content,
//             start_position: *current_position,
//             end_position: *current_position + content.len(),
//             html_tag: Some(tag_name.to_string()),
//         };
        
//         *current_position += content.len();
        
//         Ok(section)
//     }
    
//     async fn collect_content_until_next_heading(&self, start_element: ElementRef) -> Result<String> {
//         let mut content = String::new();
        
//         // Итерируемся по следующим элементам
//         let mut current_node = start_element.next_sibling();
        
//         while let Some(node) = current_node {
//             // Проверяем, не является ли элемент заголовком или новой секцией
//             if let Some(element) = node.as_element() {
//                 let tag_name = element.name();
                
//                 // Стоп-условия
//                 if tag_name.starts_with('h') && tag_name.len() == 2 {
//                     break;
//                 }
                
//                 if matches!(tag_name, "section" | "article" | "div.section" | "div.chapter") {
//                     break;
//                 }
                
//                 // Обрабатываем специальные элементы
//                 if self.extract_tables && tag_name == "table" {
//                     let table_content = self.extract_table_content(element).await?;
//                     content.push_str(&table_content);
//                     content.push_str("\n\n");
//                 } else if self.extract_code_blocks && tag_name == "pre" {
//                     let code_content = self.extract_code_content(element).await?;
//                     content.push_str(&code_content);
//                     content.push_str("\n\n");
//                 } else {
//                     // Обычный текст
//                     let text = node.text().collect::<String>();
//                     if !text.trim().is_empty() {
//                         content.push_str(&text);
//                         content.push(' ');
//                     }
//                 }
//             } else if let Some(text) = node.as_text() {
//                 // Текстовый узел
//                 let text_str = text.text.trim();
//                 if !text_str.is_empty() {
//                     content.push_str(text_str);
//                     content.push(' ');
//                 }
//             }
            
//             current_node = node.next_sibling();
//         }
        
//         Ok(content.trim().to_string())
//     }
    
//     async fn extract_table_content(&self, table_element: &scraper::element_ref::ElementRef) -> Result<String> {
//         let mut table_text = String::new();
        
//         // Извлекаем строки
//         if let Ok(row_selector) = Selector::parse("tr") {
//             for row in table_element.select(&row_selector) {
//                 let mut row_text = String::new();
                
//                 // Ячейки
//                 if let Ok(cell_selector) = Selector::parse("td, th") {
//                     for cell in row.select(&cell_selector) {
//                         let cell_text = cell.text().collect::<String>().trim().to_string();
//                         if !cell_text.is_empty() {
//                             row_text.push_str(&cell_text);
//                             row_text.push_str(" | ");
//                         }
//                     }
//                 }
                
//                 if !row_text.is_empty() {
//                     table_text.push_str(&row_text.trim_end_matches(" | "));
//                     table_text.push('\n');
//                 }
//             }
//         }
        
//         Ok(format!("[TABLE]\n{}\n[/TABLE]", table_text.trim()))
//     }
    
//     async fn extract_code_content(&self, pre_element: &scraper::element_ref::ElementRef) -> Result<String> {
//         let code_text = pre_element.text().collect::<String>();
        
//         // Пытаемся определить язык программирования
//         let language = pre_element.value()
//             .attr("class")
//             .and_then(|classes| {
//                 classes.split_whitespace()
//                     .find(|c| c.starts_with("language-"))
//                     .map(|c| &c[9..])
//             })
//             .unwrap_or("text");
        
//         Ok(format!("[CODE lang={}]\n{}\n[/CODE]", language, code_text.trim()))
//     }
    
//     fn build_outline(&self, sections: &[DocumentSection]) -> Vec<OutlineItem> {
//         let mut outline = Vec::new();
//         let mut stack: Vec<OutlineItem> = Vec::new();
        
//         for (index, section) in sections.iter().enumerate() {
//             let item = OutlineItem {
//                 id: format!("section_{}", index),
//                 title: section.heading.clone(),
//                 level: section.level as usize,
//                 children: Vec::new(),
//                 section_index: Some(index),
//             };
            
//             // Находим правильное место в иерархии
//             while let Some(parent) = stack.last() {
//                 if parent.level < item.level {
//                     break;
//                 }
//                 stack.pop();
//             }
            
//             if let Some(parent) = stack.last_mut() {
//                 parent.children.push(item.clone());
//             } else {
//                 outline.push(item.clone());
//             }
            
//             stack.push(item);
//         }
        
//         outline
//     }
    
//     fn extract_title(&self, document: &Html) -> Option<String> {
//         // Пробуем несколько источников заголовка
//         let selectors = [
//             Selector::parse("title").ok(),
//             Selector::parse("h1").ok(),
//             Selector::parse("meta[property='og:title']").ok(),
//             Selector::parse("meta[name='twitter:title']").ok(),
//         ];
        
//         for selector_opt in selectors.iter().flatten() {
//             if let Some(element) = document.select(selector_opt).next() {
//                 let title = match element.value().name() {
//                     "meta" => element.value().attr("content"),
//                     _ => Some(element.text().collect::<String>().trim()),
//                 };
                
//                 if let Some(title) = title {
//                     if !title.is_empty() {
//                         return Some(title.to_string());
//                     }
//                 }
//             }
//         }
        
//         None
//     }
    
//     async fn extract_metadata(&self, document: &Html, url: &str) -> Result<DocumentMetadata> {
//         let mut metadata = DocumentMetadata {
//             source_url: url.to_string(),
//             title: self.extract_title(document).unwrap_or_default(),
//             chunk_index: 0,
//             total_chunks: 0,
//             tags: Vec::new(),
//             timestamp: chrono::Utc::now().to_rfc3339(),
//             author: None,
//             description: None,
//             language: None,
//             categories: Vec::new(),
//             word_count: 0,
//             reading_time_minutes: 0,
//         };
        
//         // Извлекаем мета-теги
//         if let Ok(meta_selector) = Selector::parse("meta") {
//             for element in document.select(&meta_selector) {
//                 if let (Some(name), Some(content)) = (element.value().attr("name"), element.value().attr("content")) {
//                     match name.to_lowercase().as_str() {
//                         "author" => metadata.author = Some(content.to_string()),
//                         "description" => metadata.description = Some(content.to_string()),
//                         "keywords" => {
//                             metadata.tags.extend(
//                                 content.split(',')
//                                     .map(|s| s.trim().to_string())
//                                     .filter(|s| !s.is_empty())
//                             );
//                         }
//                         "language" => metadata.language = Some(content.to_string()),
//                         "category" => metadata.categories.push(content.to_string()),
//                         _ => {}
//                     }
//                 }
                
//                 // Open Graph и Twitter мета-теги
//                 if let (Some(property), Some(content)) = (element.value().attr("property"), element.value().attr("content")) {
//                     match property {
//                         "og:author" => metadata.author.get_or_insert(content.to_string()),
//                         "og:description" => metadata.description.get_or_insert(content.to_string()),
//                         "og:locale" => metadata.language.get_or_insert(content.to_string()),
//                         "article:tag" => metadata.tags.push(content.to_string()),
//                         _ => {}
//                     };
//                 }
//             }
//         }
        
//         // Считаем слова и время чтения
//         let body_text = document
//             .select(&Selector::parse("body").unwrap())
//             .next()
//             .map(|e| e.text().collect::<String>())
//             .unwrap_or_default();
        
//         metadata.word_count = body_text.split_whitespace().count();
//         metadata.reading_time_minutes = (metadata.word_count as f32 / 200.0).ceil() as u32; // 200 слов в минуту
        
//         Ok(metadata)
//     }
// }
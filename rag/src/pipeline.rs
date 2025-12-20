// pub struct EnhancedDocumentPipeline {
//     structure_parser: DocumentStructureParser,
//     embedder: LongContextEmbedder,
//     qdrant: QdrantStorage,
// }

// impl EnhancedDocumentPipeline {
//     pub async fn process_with_structure(&self, html: &str, url: &str) -> Result<Vec<DocumentChunk>> {
//         // 1. Парсим структуру документа
//         let structure = self.structure_parser.parse_html(html, url).await?;
        
//         // 2. Создаем чанки с учетом структуры
//         let mut all_chunks = Vec::new();
        
//         for (section_index, section) in structure.sections.iter().enumerate() {
//             // Разбиваем секцию на чанки если она слишком длинная
//             let section_chunks = if section.content.len() > self.structure_parser.max_section_length * 4 {
//                 // Примерная эвристика: 4 символа на токен
//                 self.split_section_into_chunks(section, section_index).await?
//             } else {
//                 // Создаем один чанк для секции
//                 vec![self.create_chunk_from_section(section, section_index, 0).await?]
//             };
            
//             all_chunks.extend(section_chunks);
//         }
        
//         // 3. Генерируем эмбеддинги
//         let texts: Vec<String> = all_chunks.iter().map(|c| c.content.clone()).collect();
//         let embeddings = self.embedder.embed_batch_long(&texts, false).await?;
        
//         // 4. Обогащаем чанки эмбеддингами и метаданными
//         for (i, chunk) in all_chunks.iter_mut().enumerate() {
//             chunk.embedding = embeddings[i].clone();
            
//             // Добавляем структурные метаданные
//             chunk.metadata.section_path = Some(
//                 self.get_section_path(&structure.outline, chunk.metadata.section_id.as_ref())
//             );
//             chunk.metadata.hierarchy_level = Some(
//                 self.get_hierarchy_level(&structure.outline, chunk.metadata.section_id.as_ref())
//             );
//         }
        
//         // 5. Сохраняем в Qdrant
//         self.qdrant.upsert_chunks_with_hierarchy(all_chunks.clone()).await?;
        
//         Ok(all_chunks)
//     }
    
//     async fn split_section_into_chunks(
//         &self,
//         section: &DocumentSection,
//         section_index: usize,
//     ) -> Result<Vec<DocumentChunk>> {
//         let mut chunks = Vec::new();
        
//         // Используем семантическое разбиение внутри секции
//         // Например, по параграфам
//         let paragraphs: Vec<&str> = section.content.split("\n\n").collect();
//         let mut current_chunk = String::new();
//         let mut chunk_index = 0;
        
//         for paragraph in paragraphs {
//             // Если добавление параграфа превысит лимит, создаем новый чанк
//             if current_chunk.len() + paragraph.len() > self.structure_parser.max_section_length * 4
//                 && !current_chunk.is_empty()
//             {
//                 chunks.push(self.create_chunk_from_section(
//                     section,
//                     section_index,
//                     chunk_index,
//                 ).await?);
                
//                 current_chunk.clear();
//                 chunk_index += 1;
//             }
            
//             if !current_chunk.is_empty() {
//                 current_chunk.push_str("\n\n");
//             }
//             current_chunk.push_str(paragraph);
//         }
        
//         // Добавляем последний чанк
//         if !current_chunk.is_empty() {
//             chunks.push(self.create_chunk_from_section(
//                 section,
//                 section_index,
//                 chunk_index,
//             ).await?);
//         }
        
//         Ok(chunks)
//     }
    
//     async fn create_chunk_from_section(
//         &self,
//         section: &DocumentSection,
//         section_index: usize,
//         chunk_index: usize,
//     ) -> Result<DocumentChunk> {
//         Ok(DocumentChunk {
//             id: format!("section_{}_chunk_{}", section_index, chunk_index),
//             content: section.content.clone(),
//             metadata: DocumentMetadata {
//                 source_url: "".to_string(), // Заполнится позже
//                 title: section.heading.clone(),
//                 chunk_index,
//                 total_chunks: 1, // Обновится позже
//                 tags: Vec::new(),
//                 timestamp: chrono::Utc::now().to_rfc3339(),
//                 section_id: Some(format!("section_{}", section_index)),
//                 parent_chunk_id: None,
//                 section_path: None,
//                 author: None,
//                 description: None,
//                 language: None,
//                 categories: Vec::new(),
//                 word_count: section.content.split_whitespace().count(),
//                 token_count: 0, // Рассчитается при токенизации
//                 reading_time_minutes: 0,
//                 document_type: None,
//                 contains_tables: false, // Можно определить по контенту
//                 contains_code: false,
//                 contains_images: false,
//                 start_token: None,
//                 end_token: None,
//                 hierarchy_level: Some(section.level),
//             },
//             embedding: Vec::new(),
//         })
//     }
    
//     fn get_section_path(&self, outline: &[OutlineItem], section_id: Option<&String>) -> Vec<String> {
//         let mut path = Vec::new();
        
//         if let Some(section_id) = section_id {
//             self.find_path_in_outline(outline, section_id, &mut path);
//         }
        
//         path
//     }
    
//     fn find_path_in_outline(&self, items: &[OutlineItem], target_id: &str, path: &mut Vec<String>) -> bool {
//         for item in items {
//             if item.id == target_id {
//                 path.push(item.title.clone());
//                 return true;
//             }
            
//             if self.find_path_in_outline(&item.children, target_id, path) {
//                 path.insert(0, item.title.clone());
//                 return true;
//             }
//         }
        
//         false
//     }
// }
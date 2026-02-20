mod logger;
mod error;
mod document;
mod search;
mod service;
mod working_models;
use serde::{Deserialize, Serialize};
use scraper::Node;
use tracing::info;
pub use utilites::Date;
pub use document::{Document, DocumentCard, Collection};
pub use error::Error;
pub use service::{Service};

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use embedding::{Model};
    use tracing::{debug, info};
    use utilites::Date;

    // #[tokio::test]
    // async fn test_converter()
    // {
    //     logger::init();
    //     let converter = HtmlConverter{};
    //     let result = 
    //         systema_client::SystemaClient::get_document(
    //             Date::new_date(31, 07, 2025),
    //             "287-ФЗ", converter).await.unwrap();  
    //     let emb_cfg = Arc::new(EmbeddingConfiguration::default());
    //     let mut model = embedding::RetriverModel::new(emb_cfg).await.unwrap();
    //     model.load_model().await.unwrap();
    //     let mut chunks = Vec::with_capacity(result.node_count());
    //     info!("Ноды документы были успешно получены: {} шт.", result.node_count());
    //     let chunker = Chunker::new(&model).await.unwrap();
    //     for node in &result
    //     {
    //         //бьем текст на куски тут и для каждого создаем чанку
    //         let splitted = chunker.split_text(node.converted_content()).await.unwrap();
    //         for text in splitted
    //         {
    //             let chunk = Chunk
    //             {
    //                 publication_url: result.publication_url().to_owned(),
    //                 document_url: format!("http://actual.pravo.gov.ru/list.html#hash={}", result.hash()),
    //                 title: result.title().to_owned(),
    //                 number: result.number().to_owned(),
    //                 sign_date: result.sign_date().to_owned(),
    //                 hash: result.hash().to_owned(),
    //                 path: result.find_all_parents_as_str(&node),
    //                 links_hashes: node.links_hashes().cloned(),
    //                 content: text.content,
    //                 embeddings: None,
    //                 meta: Some(ChunkMeta
    //                 {
    //                     chunk_index: text.chunk_index,
    //                     token_count: text.token_count
    //                 })
    //             };
    //             chunks.push(chunk);
    //         }
    //     }
       
    //     for chunk in &chunks
    //     {
           
    //         debug!("chunk created: {:#?}", chunk);
    //     }
    // }

    // #[tokio::test]
    // async fn test_qdrant()
    // {
    //     logger::init();
    //     let emb_cfg = Arc::new(EmbeddingConfiguration::default());
    //     let retriver = embedding::RetriverModel::new(emb_cfg.clone()).await.unwrap();
    //     let reranker_client = embedding::BgeReranker::new(emb_cfg).await.unwrap();
    //     let qconfig = QdrantConfig
    //     {
    //         url: "http://localhost:6334".to_owned(),
    //         collection_name: "test_collection".to_owned(),
    //         distance: database::Distance::Cosine
    //     };
       
    //     let date = Date::new_date(31, 07, 2025);
    //     let number = "287-ФЗ";  
    //     let chunks = super::chunks::get_chunks(date, number, &retriver).await.unwrap();
    //     let qdrant = database::QdrantManager::new(qconfig, &retriver, &reranker_client).await.unwrap();
    //     let _ = qdrant.ensure_collection().await.unwrap();
    //     let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    //     tokio::spawn(
    //     async move 
    //     {
    //         while let Some(m) = receiver.recv().await
    //         {
    //             info!("{:?}", m);
    //         }
    //     });
        
    //     let res = qdrant.add_chunks_to_qdrant(chunks, sender).await;
    //     debug!("{:?}", res);
    //     assert!(res.is_ok(), "записи не добавлены в базу данных");

    // }

    
}

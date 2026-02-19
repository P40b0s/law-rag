use database::SearchResult;





#[cfg(test)]
mod tests
{
    use std::sync::Arc;
    use embedding::{ Generator, GeneratorLlama1b, Model};
    use rag_core::{Embedder, EmbeddingConfiguration, QdrantConfiguration};
    use tracing::{debug, info};

    use crate::logger;

    #[tokio::test]
    async fn search()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let qd_cfg = Arc::new(QdrantConfiguration::default());
        let mut retriver = embedding::RetriverModel::new(emb_cfg.clone()).await.unwrap();
        retriver.load_model().await.unwrap();
        let mut reranker_client = embedding::BgeReranker::new(emb_cfg.clone()).await.unwrap();
        reranker_client.load_model().await.unwrap();
        let qdrant = database::QdrantManager::new(qd_cfg, emb_cfg).unwrap();
        // Пример поиска похожих документов
        let query = "Как взыскивается задолженность с налогоплательщика?";
        let collection = qdrant.get_collections_names().await.unwrap().first().unwrap().to_owned();
        let embeddings = retriver.generate_embeddings(query).await.unwrap();
        //let emb = emb_client.generate_embeddings(&[texts]).await.unwrap();
        let similar = qdrant.vector_search_raw(embeddings, &collection,   5, None).await.unwrap();
        let _ = retriver.unload_model();
        let _ = reranker_client.unload_model();
        info!("Топ-5 похожих чанков:");
        for similarity in &similar 
        {
            info!("Сходство: {:.4}", similarity.score);
            info!("Сходство после реранкинга: {:.4}", similarity.score);
            info!("Документ: {}", similarity.payload.document_uri);
            info!("Чанк #{}: {}", similarity.payload.document_number, &similarity.payload.text);
            info!("---");
        }
    }


    #[tokio::test]
    async fn load()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let mut retriver = embedding::RetriverModel::new(emb_cfg.clone()).await.unwrap();
        retriver.load_model().await.unwrap();
        let mut reranker_client = embedding::BgeReranker::new(emb_cfg.clone()).await.unwrap();
        reranker_client.load_model().await.unwrap();
        let reranker_model_name = reranker_client.name();
        let retriver_model_name = retriver.name();
        info!("retriver model: {} reranker model: {}", retriver_model_name.as_ref(), reranker_model_name.as_ref());
        
    }
}

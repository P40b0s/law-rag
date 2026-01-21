#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use database::QdrantConfig;
    use embedding::EmbeddingConfiguration;
    use tracing::info;

    use crate::logger;

    #[tokio::test]
    async fn test_search()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let emb_client = embedding::Embeddings::new(emb_cfg.clone()).await.unwrap();
        let reranker_client = embedding::BgeReranker::new(emb_cfg).await.unwrap();
        let qconfig = QdrantConfig
        {
            url: "http://localhost:6334".to_owned(),
            collection_name: "test_collection".to_owned(),
            distance: database::Distance::Cosine
        };
        let qdrant = database::QdrantManager::new(qconfig, emb_client, reranker_client).await.unwrap();
        // Пример поиска похожих документов
        let query = "Как взыскивается задолженность с налогоплательщика?";
        //let emb = emb_client.generate_embeddings(&[texts]).await.unwrap();
        let similar = qdrant.semantic_search(query, 5, None).await.unwrap();
        
        info!("Топ-5 похожих чанков:");
        for similarity in similar 
        {
            info!("Сходство: {:.4}", similarity.payload.score);
            info!("Сходство после реранкинга: {:.4}", similarity.score);
            info!("Документ: {}", similarity.payload.payload.document_uri);
            info!("Чанк #{}: {}", similarity.payload.payload.document_number, &similarity.payload.payload.text);
            info!("---");
        }
    }
}
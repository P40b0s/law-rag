use database::SearchResult;
use embedding::RerankResult;





#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use database::QdrantConfig;
    use embedding::{EmbeddingConfiguration, Generator, ModelPrompt, Model};
    use tracing::{debug, info};

    use crate::logger;

    #[tokio::test]
    async fn test_search()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let emb_client = embedding::Embeddings::new(emb_cfg.clone()).await.unwrap();
        let reranker_client = embedding::BgeReranker::new(emb_cfg.clone()).await.unwrap();
        let qconfig = QdrantConfig
        {
            url: "http://localhost:6334".to_owned(),
            collection_name: "test_collection".to_owned(),
            distance: database::Distance::Cosine
        };
        let qdrant = database::QdrantManager::new(qconfig, &emb_client, &reranker_client).await.unwrap();
        // Пример поиска похожих документов
        let query = "Как взыскивается задолженность с налогоплательщика?";
        //let emb = emb_client.generate_embeddings(&[texts]).await.unwrap();
        let similar = qdrant.semantic_search(query, 5, 5, None).await.unwrap();
        let _ = emb_client.unload_model();
        let _ = reranker_client.unload_model();
        info!("Топ-5 похожих чанков:");
        for similarity in &similar 
        {
            info!("Сходство: {:.4}", similarity.db_object.score);
            info!("Сходство после реранкинга: {:.4}", similarity.score);
            info!("Документ: {}", similarity.db_object.payload.document_uri);
            info!("Чанк #{}: {}", similarity.db_object.payload.document_number, &similarity.db_object.payload.text);
            info!("---");
        }
        //TODO добавить генератор для вывода сообщения пользователю
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        // tokio::spawn(
        //     async move 
        //     {
        //         while let Some(output) = receiver.recv().await
        //         {
        //             info!("{}", output);
        //         }
        //     }
        // );
        let mut generator = Generator::load(emb_cfg).unwrap().load_model().await.unwrap();
        std::thread::spawn(move ||
            {
                let result = generator.prompt(query, similar.as_slice(), sender);
                debug!("{:?}", result);
            }
        );
        // tokio::task::spawn_blocking(move ||
        // {
           
        // });
        while let Some(output) = receiver.recv().await
        {
            info!("{}", output);
        }
        
    }
}

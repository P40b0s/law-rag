use std::{collections::{self, HashMap}, fmt::Display, ops::Deref, sync::{Arc, atomic::AtomicBool}};
use anyhow::{Context, anyhow};
use database::{QdrantConfig, QdrantManager, SearchResult};
use embedding::{BgeReranker, EmbeddingConfiguration, Generator, GeneratorLlama1b, GeneratorLlama8b, Model, RetriverModel};
use rag_core::{Chunk, RerankResult, ServiceStatus};
use serde::Serialize;
use tokio::sync::{RwLock, mpsc::{UnboundedReceiver, UnboundedSender}};
use tracing::{debug, error, info, warn};
use utilites::Date;
pub type LlmModel = GeneratorLlama1b;
use crate::{Collection, Error};

#[derive(Debug, Serialize)]
pub struct ModelsState
{
    retriver: bool,
    generator: bool,
    system_prompt: String,
    model_size: usize
}
pub struct Service
{
    embedding_config: Arc<EmbeddingConfiguration>,
    retriver: RwLock<RetriverModel>,
    retriver_dimension: usize,
    reranker: RwLock<BgeReranker>,
    generator: RwLock<LlmModel>,

}

impl Service
{
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self, Error>
    {
        let generator = LlmModel::load(Arc::clone(&emb_cfg))
            .inspect_err(|e| error!("Error when loading generator `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "generator".to_owned(), source: e })?;
        let retriver = RetriverModel::new(Arc::clone(&emb_cfg)).await
            .inspect_err(|e| error!("Error when loading retriver model `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "retriver".to_owned(), source: e })?;
        let retriver_dimension = retriver.dimension();
        let reranker = BgeReranker::new(Arc::clone(&emb_cfg)).await
            .inspect_err(|e| error!("Error when loading reranker model `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "reranker".to_owned(), source: e })?;
        let slf = Self
        {
            embedding_config: emb_cfg,
            retriver: RwLock::new(retriver),
            reranker: RwLock::new(reranker),
            generator: RwLock::new(generator),
            retriver_dimension
        };
       Ok(slf)
    }
    pub async fn models_state(&self) -> ModelsState
    {
        let retriver_lock = self.retriver.read().await;
        let retriver = retriver_lock.model_is_loaded();
        drop(retriver_lock);
        let reranker_lock = self.reranker.read().await;
        let reranker = reranker_lock.model_is_loaded();
        drop(reranker_lock);
        let gen_lock = self.generator.read().await;
        let generator = gen_lock.model_is_loaded();
        let system_prompt = gen_lock.get_system_prompt().to_owned();
        let model_size = gen_lock.get_size_in_bytes();
        ModelsState
        {
            retriver: retriver && reranker,
            generator,
            system_prompt,
            model_size
        }
    }

    pub async fn get_retriver(&self) -> tokio::sync::RwLockReadGuard<'_, RetriverModel>
    {
        self.retriver.read().await
    }
    pub async fn get_retriver_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, RetriverModel>
    {
        self.retriver.write().await
    }
    pub async fn load_generator_model(&self) -> Result<ModelsState, Error>
    {
        let mut gen_lock = self.generator.write().await;
        if gen_lock.model_is_loaded()
        {
            warn!("Generator model already loaded");
            drop(gen_lock);
            let state = self.models_state().await;
            Ok(state)
        }
        else
        {
            gen_lock.load_model().await
                .map_err(|e| Error::ModelLoadError { model: "generator".to_owned(), source: e })?;
            drop(gen_lock);
            let state = self.models_state().await;
            Ok(state)
        }
    }
    fn qdrant_config(&self, collection_name: &str) -> QdrantConfig
    {
        QdrantConfig
        {
            url: "http://localhost:6334".to_owned(),
            collection_name: collection_name.to_owned(),
            distance: database::Distance::Cosine
        }
    }
    ///create collection if not exists
    pub async fn create_collection(&self, collection_name: &str) -> Result<(), Error>
    {
        let cfg = self.qdrant_config(collection_name);
        let retriver = self.retriver.read().await;
        let qm = QdrantManager::new(cfg, retriver.dimension())?;
        qm.ensure_collection().await?;
        Ok(())
    }
    //std::thread::spawn inside, may be long operation
    //progress messages and errors sends via sender
    //uses separate OS thread to avoid blocking tokio runtime with CPU-bound embedding generation
    pub async fn add_chunks_to_qdrant(self: Arc<Self>, chunks: Vec<Chunk>, collection_name: &str) -> Result<UnboundedReceiver<ServiceStatus>, Error>
    {
        let cfg = self.qdrant_config(collection_name);
        let drop_service = self;
        let service = Arc::clone(&drop_service);
        let _ = service.check_load_embedding_models().await?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::runtime::Handle::current();
        std::thread::spawn(move ||
        {
            handle.block_on(async move
            {
                let retriver = service.retriver.read().await;
                let dimension = retriver.dimension();
                let qm = QdrantManager::new(cfg, dimension)
                    .inspect_err(|e| error!("Error creating QdrantManager: {}", e));

                if let Ok(qm) = qm
                {
                    let check_collection = qm.ensure_collection().await;
                    if let Err(e) = check_collection
                    {
                        let _ = sender.send(ServiceStatus::qdrant_error(e));
                    }
                    else
                    {
                        //auto send error to client
                        let _added = qm.add_chunks_to_qdrant(chunks, &retriver, sender).await;
                    }
                }
                else
                {
                    let _ = sender.send(ServiceStatus::qdrant_error(qm.err().unwrap()));

                }
            });
        });
        Ok(receiver)

    }

    async fn check_load_embedding_models(&self) -> Result<(), Error>
    {
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        if !reranker.model_is_loaded()
        {
            return Err(Error::RerankerModelNotLoad)
        }
        if !retriver.model_is_loaded()
        {
            return Err(Error::RetriverModelNotLoad)
        }
        Ok(())
    }
    
    ///может потратить много времени на реранкинг, пока не буду канал выводить для статуса
    pub async fn search(&self, query: &str, limit: usize, reranker_limit: usize, collection_name: &str) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        let cfg = self.qdrant_config(collection_name);
        let _ = self.check_load_embedding_models().await?;
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        let qm = QdrantManager::new(cfg, retriver.dimension())
            .inspect_err(|e| error!("Error creating QdrantManager: {}", e))?;
        let result = qm.semantic_search(&query, limit, reranker_limit, &retriver, &reranker, None).await?;
        Ok(result)
    }

    /// Поиск по нескольким коллекциям с общим reranker_limit
    ///
    /// Собирает результаты из всех указанных коллекций, сортирует по score
    /// и возвращает топ reranker_limit результатов по всем коллекциям
    /// Search across multiple collections using CPU-bound embedding generation on separate thread
    /// to avoid blocking the tokio runtime
    pub async fn search_multiple_collections(
        self: Arc<Self>,
        query: &str,
        limit: usize,
        reranker_limit: usize,
        collection_names: &[&str]
    ) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        if collection_names.is_empty() 
        {
            return Ok(Vec::new());
        }

        let _ = self.check_load_embedding_models().await?;
        let query = query.to_string();
        let collection_names: Vec<String> = collection_names.iter().map(|s| s.to_string()).collect();
        let service = Arc::clone(&self);
        //let retriver = self.retriver.clone();
        //let reranker = self.reranker.clone();
        let handle = tokio::runtime::Handle::current();
        // Run CPU-bound embedding generation on separate thread
        let result = std::thread::spawn(move || 
        {
            handle.block_on(async move 
            {
                let retriver = service.retriver.read().await;
                let reranker = service.reranker.read().await;
                let mut all_results = Vec::new();
                let query_embeddings = retriver.generate_embeddings(&[&query]).await
                    .map_err(|e| Error::EmbeddingsError {source: embedding::Error::RerankerError { source: e }})
                    .inspect_err(|e| error!("Create embeddings error: {}", e))
                    .unwrap_or(Vec::new());
                for collection_name in &collection_names
                {
                    let cfg = service.qdrant_config(collection_name);

                    let qm = QdrantManager::new(cfg, retriver.dimension())
                        .inspect_err(|e| error!("Error creating QdrantManager for collection {}: {}", collection_name, e))?;
                    let _check_collection = qm.ensure_collection().await
                        .map_err(|e| Error::DatabaseError(e));

                    // Для каждой коллекции используем limit, но не ограничиваем reranker_limit
                    // т.к. финальный лимит будет применен после объединения всех результатов
                    match qm.semantic_search_with_embeddings(&query, limit, limit, query_embeddings.clone(), &reranker, None).await
                    {
                        Ok(mut results) =>
                        {
                            info!("Found {} results in collection {}", results.len(), collection_name);
                            all_results.append(&mut results);
                        },
                        Err(e) =>
                        {
                            warn!("Error searching in collection {}: {}", collection_name, e);
                            // Продолжаем поиск в других коллекциях даже если одна из них вернула ошибку
                        }
                    }
                }

                // Сортируем все результаты по score в порядке убывания (большие score первыми)
                all_results.sort_by(|a, b|
                {
                    b.score.partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // Берем только топ reranker_limit результатов
                all_results.truncate(reranker_limit);

                info!("Returning {} top results from {} collections", all_results.len(), collection_names.len());
                Ok(all_results)
            })
        }).join();
        let result = match result
        {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(Error::ThreadError(anyhow!("Thread panicked: {:?}", e))),
        };

        Ok(result)
    }

    /// Ранжирует коллекции по релевантности к запросу пользователя
    ///
    /// Использует reranker для оценки семантической близости описаний коллекций
    /// к запросу пользователя и возвращает отсортированный список коллекций.
    ///
    /// # Аргументы
    /// * `query` - Запрос пользователя
    /// * `collections` - Список коллекций для ранжирования
    /// * `min_score` - Минимальный порог релевантности (0.0 - 1.0). Коллекции с меньшим score будут отфильтрованы
    ///
    /// # Возвращает
    /// Вектор коллекций, отсортированных по релевантности (от большего к меньшему)
    ///
    /// # Пример
    /// ```ignore
    /// let relevant_collections = service.rank_collections_by_relevance(
    ///     "законы о туризме",
    ///     collections,
    ///     0.3  // Порог релевантности 30%
    /// ).await?;
    /// ```
    pub async fn rank_collections_by_relevance(
        &self,
        query: &str,
        collections: Vec<Collection>,
        min_score: f32
    ) -> Result<Vec<RerankResult<Collection>>, Error>
    {
        if collections.is_empty() 
        {
            return Ok(Vec::new());
        }

        // Проверяем, что reranker загружен
        let reranker = self.reranker.read().await;
        if !reranker.model_is_loaded() 
        {
            return Err(Error::RerankerModelNotLoad);
        }

        info!("Ranking {} collections for query: {}", collections.len(), query);

        // Создаем описания коллекций для reranker
        // Включаем название, описание и ключевые слова для более точной оценки релевантности
        let descriptions: Vec<String> = collections
            .iter()
            .map(|c| {
                let keywords_str = c.keywords.join(", ");
                format!("{}: {}. {}", c.name, c.description, keywords_str)
            })
            .collect();
        debug!("strings for reranking: {:?}", &descriptions);
        // Используем rerank для оценки релевантности
        // top_k = collections.len() чтобы получить все результаты
        let ranked_results = reranker.rerank(query, descriptions, collections.len()).await
            .map_err(|e| Error::ModelError { source: e })?;

        // Преобразуем результаты обратно в Collection с сохранением score
        let mut results_with_collections: Vec<RerankResult<Collection>> = Vec::new();
        for (idx, result) in ranked_results.iter().enumerate() 
        {
            if result.score >= min_score 
            {
                results_with_collections.push(RerankResult 
                {
                    score: result.score,
                    db_object: collections[idx].clone(),
                });
            }
        }

        info!(
            "Found {} relevant collections (min_score: {:.2})",
            results_with_collections.len(),
            min_score
        );

        // Логируем топ-3 результата для отладки
        for (i, result) in results_with_collections.iter().take(3).enumerate() 
        {
            info!(
                "  #{}: {} (score: {:.3})",
                i + 1,
                result.db_object.name,
                result.score
            );
        }

        Ok(results_with_collections)
    }

    pub async fn load_embedding_models(&self) -> Result<ModelsState, Error>
    {
        {
            let mut lock_retriver = self.retriver.write().await;
            let _ = lock_retriver.load_model().await
                .map_err(|e| Error::ModelLoadError { model: "retriver".to_owned(), source: e })?;
            let mut lock_reranker = self.reranker.write().await;
            let _ = lock_reranker.load_model().await
                .map_err(|e| Error::ModelLoadError { model: "reranker".to_owned(), source: e })?;
        }
        let state = self.models_state().await;
        Ok(state)
    }
    pub async fn unload_generator_model(&self) -> Result<ModelsState, Error>
    {
        let mut lock = self.generator.write().await;
        lock.unload_model();
        drop(lock);
        let state = self.models_state().await;
        Ok(state)
    }
    pub async fn unload_embedding_models(&self) -> Result<ModelsState, Error>
    {
        let mut lock = self.retriver.write().await;
        lock.unload_model();
        drop(lock);
        let mut lock = self.reranker.write().await;
        lock.unload_model();
        drop(lock);
        let state = self.models_state().await;
        Ok(state)
    }

    pub async fn load_all(&self) -> Result<ModelsState, Error>
    {
        let _generator = self.load_generator_model().await?;
        let last_state = self.load_embedding_models().await?;
        Ok(last_state)
    }

    pub async fn delete_document_from_qdrant(&self, hash: &str, collection_name: &str) -> Result<(), Error>
    {
        let qm = QdrantManager::new(self.qdrant_config(collection_name), self.retriver_dimension)
            .inspect_err(|e| error!("Error creating QdrantManager: {}", e))?;
        let status = qm.delete_document(hash).await?;
        info!("Document with hash {} deleted from Qdrant, status: {:?}", hash, status);
        Ok(())
    }


    pub async fn genereate_response(self: Arc<Self>, query: &str, context: Vec<RerankResult<SearchResult>>) -> Result<UnboundedReceiver<String>, Error>
    {
        let service = self;
        let generator = service.generator.read().await;
        if !generator.model_is_loaded()
        {
            return Err(Error::GeneratorModelNotLoad);
        }
        drop(generator);
        let task_service = Arc::clone(&service);
        let query = query.to_owned();
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        std::thread::spawn(move ||
        {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut generator = task_service.generator.write().await;
                let result = generator.prompt(&query, context.as_slice(), sender);
                debug!("{:?}", result);
             });
        });
        Ok(receiver)
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use database::QdrantConfig;
    use embedding::EmbeddingConfiguration;
    use tracing::info;
    use utilites::Date;

    use crate::{logger};

    //  #[tokio::test]
    // async fn service()
    // {
    //     logger::init();
    //     let emb_cfg = Arc::new(EmbeddingConfiguration::default());
    //     let service = Arc::new(super::Service::new(emb_cfg).await.unwrap());
    //     service.load_embedding_models().await.unwrap();
    //     service.init_qdrant().await.unwrap();
    //     let date = Date::new_date(31, 07, 2025);
    //     let number = "287-ФЗ";  
    //     let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    //     tokio::spawn(async move {
    //         while let Some(output) = receiver.recv().await
    //         {
    //             info!("{:?}", output);
    //         }
    //     });
    //     let chunks = service.get_chunks(date, number,sender).await.unwrap();
    //     let add_qd_clone = Arc::clone(&service);
    //     let mut receiver = add_qd_clone.add_chunks_to_qdrant(chunks).await.unwrap();
    //     while let Some(output) = receiver.recv().await
    //     {
    //         info!("{:?}", output);
    //     }
    //     let query = "Как взыскивается задолженность с налогоплательщика?";
    //     let context = service.search(query, 5, 5).await.unwrap();
    //     service.unload_embedding_models().await.unwrap();
    //     service.load_generator_model().await.unwrap();
    //     //до сюда доходит без ошибок, тест генерации отдельно проверю в другом тесте
    //     let mut receiver = service.genereate_response(query, context).await.unwrap();
    //     info!("GEN RESPONSE:");
    //     while let Some(output) = receiver.recv().await
    //     {
    //         info!("{:?}", output);
    //     }
    // }

    #[tokio::test]
    async fn search_and_gen()
    {
        use embedding::GeneratorLlama1b; // or GeneratorLlama8b

        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let service = Arc::new(super::Service::new(emb_cfg).await.unwrap());
        service.load_embedding_models().await.unwrap();
        service.create_collection("law_collection").await.unwrap();
        let query = "Как взыскивается задолженность с налогоплательщика?";
        let context = service.search(query, 5, 5, "law_collection").await.unwrap();
        service.unload_embedding_models().await.unwrap();
        service.load_generator_model().await.unwrap();
        let mut receiver = service.genereate_response(query, context).await.unwrap();
        info!("GEN RESPONSE:");
        while let Some(output) = receiver.recv().await
        {
            info!("{:?}", output);
        }
    }
    //
}

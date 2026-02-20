use std::sync::Arc;
use anyhow::anyhow;
use database::{QdrantManager, SearchResult};
use embedding::{Generator, Model};
use rag_core::{Chunk, Embedder, EmbeddingConfiguration, Encoder, GeneratorConfig, ModelsState, QdrantConfiguration, RerankResult, Reranker, ServiceStatus};
use serde::Serialize;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error, info, warn};
use crate::{Error, working_models::WorkingModels};


pub struct Service
{
    models: WorkingModels,
    qdrant_manager: QdrantManager,
    emb_conf: Arc<EmbeddingConfiguration>
}

impl Service
{
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>, qdrant_cfg: Arc<QdrantConfiguration>) -> Result<Self, Error>
    {
        let models = WorkingModels::new(emb_cfg.clone()).await?;
        let qdrant_manager = QdrantManager::new(Arc::clone(&qdrant_cfg), Arc::clone(&emb_cfg))?;
        let slf = Self
        {
            qdrant_manager,
            models,
            emb_conf: emb_cfg
        };
       Ok(slf)
    }
    pub async fn models_state(&self) -> ModelsState
    {
        let mut models_state = ModelsState::new();
        let retriver_lock = self.models.get_retriver().await;
        let retriver = retriver_lock.model_is_loaded();
        let retriver_model_name = retriver_lock.name().to_string();
        drop(retriver_lock);
        let reranker_lock = self.models.get_reranker().await;
        let reranker = reranker_lock.model_is_loaded();
        let reranker_model_name = reranker_lock.name().to_string();
        drop(reranker_lock);
        if retriver && reranker
        {
            models_state = models_state.with_retriver(retriver_model_name, reranker_model_name);
        }
        let gen_lock = self.models.get_generator().await;
        let generator = gen_lock.model_is_loaded();
        let generator_model_name = gen_lock.model_name().to_string();
        let generator_size = gen_lock.get_size_in_bytes();
        let system_prompt = gen_lock.get_system_prompt().to_owned();
        let temperature = self.emb_conf.generator_temperature;
        let uri = self.emb_conf.extended_generator_url.clone();
        drop(gen_lock);
        if generator
        {
            models_state = models_state.with_generator(generator_model_name, generator_size, system_prompt, temperature, uri);
        }
        models_state
    }

    pub async fn get_encoder(&self) -> tokio::sync::RwLockReadGuard<'_, impl Encoder>
    {
        self.models.get_encoder().await
    }
    pub async fn load_generator_model(&self) -> Result<ModelsState, Error>
    {
        let mut gen_lock = self.models.get_generator_mut().await;
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
    ///create collection if not exists
    pub async fn create_collection(&self, collection_name: &str) -> Result<(), Error>
    {
        let dimension = 
        {
            let reranker = self.models.get_reranker().await;
            reranker.dimension()
        };
        let _ = self.qdrant_manager.ensure_collection(collection_name, dimension).await
            .inspect_err(|e| error!("{}",e))?;
        Ok(())
    }
    //std::thread::spawn inside, may be long operation
    //progress messages and errors sends via sender
    //uses separate OS thread to avoid blocking tokio runtime with CPU-bound embedding generation
    pub async fn add_chunks_to_qdrant(self: Arc<Self>, chunks: Vec<Chunk>, collection_name: &str) -> Result<UnboundedReceiver<ServiceStatus>, Error>
    {
        let service = Arc::clone(&self);
        let _ = service.check_load_embedding_models().await?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::runtime::Handle::current();
        let collection_name = collection_name.to_owned();
        std::thread::spawn(move ||
        {
            handle.block_on(async move
            {
                let retriver = service.models.get_retriver().await;
                let check_collection = service.qdrant_manager.ensure_collection(&collection_name, Model::dimension(&*retriver)).await;
                if let Err(e) = check_collection
                {
                    let _ = sender.send(ServiceStatus::qdrant_error(e));
                }
                else
                {
                    //auto send error to client
                    let _added = 
                    service
                    .qdrant_manager
                    .add_chunks_to_qdrant(
                        chunks,
                        &*retriver,
                        &collection_name,
                        sender).await;
                }
            });
        });
        Ok(receiver)

    }

    async fn check_load_embedding_models(&self) -> Result<(), Error>
    {
        let reranker = self.models.get_reranker().await;
        let retriver = self.models.get_retriver().await;
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
    // pub async fn search(&self, query: &str, limit: usize, reranker_limit: usize, collection_name: &str) -> Result<Vec<RerankResult<SearchResult>>, Error>
    // {
    //     let cfg = self.qdrant_config(collection_name);
    //     let _ = self.check_load_embedding_models().await?;
    //     let reranker = self.reranker.read().await;
    //     let retriver = self.retriver.read().await;
    //     let qm = QdrantManager::new(cfg, retriver.dimension())
    //         .inspect_err(|e| error!("Error creating QdrantManager: {}", e))?;
    //     let result = qm.semantic_search(&query, limit, reranker_limit, &retriver, &reranker, None).await?;
    //     Ok(result)
    // }

    /// Поиск по нескольким коллекциям одновременно.
    ///
    /// Алгоритм:
    /// 1. Генерируем эмбеддинг запроса один раз.
    /// 2. Из каждой коллекции берём `per_collection_limit` сырых результатов (без реранкинга).
    ///    Лимит адаптивный: `max(MIN_PER_COLLECTION, final_limit * 2 / collection_count)`,
    ///    чтобы реранкер суммарно получал ~2x от желаемого финального числа чанков.
    /// 3. Один общий реранкинг по всему пулу → топ `final_limit`.
    pub async fn search(
        self: Arc<Self>,
        query: &str,
        final_limit: usize,
        //берем наверно по 20-30 с каждой коллекции
        min_per_collection: usize
    ) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        let collection_names = self.qdrant_manager.get_collections_names().await?;
        if collection_names.is_empty()
        {
            return Ok(Vec::new());
        }
        // score_threshold на уровне Qdrant уже отсеивает нерелевантные коллекции,
        // поэтому берём фиксированный лимит с каждой коллекции.
        let per_collection = min_per_collection;

        let _ = self.check_load_embedding_models().await?;
        let query = query.to_string();
        let collection_names: Vec<String> = collection_names.iter().map(|s| s.to_string()).collect();
        let service = Arc::clone(&self);
        let handle = tokio::runtime::Handle::current();

        let result = std::thread::spawn(move ||
        {
            handle.block_on(async move
            {
                let retriver = service.models.get_retriver().await;
                let reranker = service.models.get_reranker().await;

                // Эмбеддинг запроса генерируем один раз для всех коллекций.
                let query_embeddings = retriver.generate_embeddings(&query).await
                    .map_err(|e| Error::EmbeddingsError { source: embedding::Error::RerankerError { source: e } })
                    .inspect_err(|e| error!("Embeddings error: {}", e))?;

                // Сырые результаты из всех коллекций (без реранкинга).
                let mut raw: Vec<SearchResult> = Vec::new();
                for collection_name in &collection_names
                {
                    match self.qdrant_manager.vector_search_raw(query_embeddings.clone(), collection_name, per_collection, None).await
                    {
                        Ok(mut results) =>
                        {
                            info!("Collection '{}': {} raw hits (per_collection_limit={})",
                                collection_name, results.len(), per_collection);
                            raw.append(&mut results);
                        }
                        Err(e) => warn!("Search error in '{}': {}", collection_name, e),
                    }
                }

                info!("Total raw candidates: {} from {} collections → reranking to top {}",
                    raw.len(), collection_names.len(), final_limit);

                // Один общий реранкинг — возвращает топ final_limit.
                let reranked = reranker.rerank(&query, raw, final_limit).await
                    .map_err(|e| Error::ModelError { source: e })?;

                Ok(reranked)
            })
        }).join();

        match result
        {
            Ok(Ok(res)) => Ok(res),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(Error::ThreadError(anyhow!("Thread panicked: {:?}", e))),
        }
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
    // pub async fn rank_collections_by_relevance(
    //     &self,
    //     query: &str,
    //     collections: Vec<Collection>,
    //     min_score: f32
    // ) -> Result<Vec<RerankResult<Collection>>, Error>
    // {
    //     if collections.is_empty() 
    //     {
    //         return Ok(Vec::new());
    //     }

    //     // Проверяем, что reranker загружен
    //     let reranker = self.reranker.read().await;
    //     if !reranker.model_is_loaded() 
    //     {
    //         return Err(Error::RerankerModelNotLoad);
    //     }

    //     info!("Ranking {} collections for query: {}", collections.len(), query);

    //     // Создаем описания коллекций для reranker
    //     // Включаем название, описание и ключевые слова для более точной оценки релевантности
    //     let descriptions: Vec<String> = collections
    //         .iter()
    //         .map(|c| {
    //             let keywords_str = c.keywords.join(", ");
    //             format!("{}: {}. {}", c.name, c.description, keywords_str)
    //         })
    //         .collect();
    //     debug!("strings for reranking: {:?}", &descriptions);
    //     // Используем rerank для оценки релевантности
    //     // top_k = collections.len() чтобы получить все результаты
    //     let ranked_results = reranker.rerank(query, descriptions, collections.len()).await
    //         .map_err(|e| Error::ModelError { source: e })?;

    //     // Преобразуем результаты обратно в Collection с сохранением score
    //     let mut results_with_collections: Vec<RerankResult<Collection>> = Vec::new();
    //     for (idx, result) in ranked_results.iter().enumerate() 
    //     {
    //         if result.score >= min_score 
    //         {
    //             results_with_collections.push(RerankResult 
    //             {
    //                 score: result.score,
    //                 db_object: collections[idx].clone(),
    //             });
    //         }
    //     }

    //     info!(
    //         "Found {} relevant collections (min_score: {:.2})",
    //         results_with_collections.len(),
    //         min_score
    //     );

    //     // Логируем топ-3 результата для отладки
    //     for (i, result) in results_with_collections.iter().take(3).enumerate() 
    //     {
    //         info!(
    //             "  #{}: {} (score: {:.3})",
    //             i + 1,
    //             result.db_object.name,
    //             result.score
    //         );
    //     }

    //     Ok(results_with_collections)
    // }

    pub async fn load_embedding_models(&self) -> Result<ModelsState, Error>
    {
        {
            let mut lock_retriver = self.models.get_retriver_mut().await;
            let _ = lock_retriver.load_model().await
                .map_err(|e| Error::ModelLoadError { model: "retriver".to_owned(), source: e })?;
            let mut lock_reranker = self.models.get_reranker_mut().await;
            let _ = lock_reranker.load_model().await
                .map_err(|e| Error::ModelLoadError { model: "reranker".to_owned(), source: e })?;
        }
        let state = self.models_state().await;
        Ok(state)
    }
    pub async fn unload_generator_model(&self) -> Result<ModelsState, Error>
    {
        let mut lock = self.models.get_generator_mut().await;
        lock.unload_model();
        drop(lock);
        let state = self.models_state().await;
        Ok(state)
    }
    pub async fn unload_embedding_models(&self) -> Result<ModelsState, Error>
    {
        let mut lock = self.models.get_retriver_mut().await;
        lock.unload_model();
        drop(lock);
        let mut lock = self.models.get_reranker_mut().await;
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

    pub async fn delete_document_from_qdrant(&self, hash: &str) -> Result<(), Error>
    {
        let status = self.qdrant_manager.delete_document(hash).await?;
        info!("Document with hash {} deleted from Qdrant, status: {:?}", hash, status);
        Ok(())
    }


    pub async fn genereate_response(self: Arc<Self>, query: &str, context: Vec<RerankResult<SearchResult>>) -> Result<UnboundedReceiver<String>, Error>
    {
        let service = self;
        let generator = service.models.get_generator().await;
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
                let mut generator = task_service.models.get_generator_mut().await;
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
    use rag_core::{EmbeddingConfiguration, QdrantConfiguration};
    use tracing::info;
    use crate::logger;

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
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let qd_cfg = Arc::new(QdrantConfiguration::default());
        let service = Arc::new(super::Service::new(emb_cfg, qd_cfg).await.unwrap());
        service.load_embedding_models().await.unwrap();
        service.create_collection("law_collection").await.unwrap();
        let query = "Как взыскивается задолженность с налогоплательщика?";
        let search_service = service.clone();
        let context = search_service.search(query, 20, 20).await.unwrap();
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

use std::{collections::HashMap, ops::Deref, sync::{Arc, atomic::AtomicBool}};
use anyhow::Context;
use database::{QdrantConfig, QdrantManager, SearchResult};
use embedding::{BgeReranker, EmbeddingConfiguration, Generator, Model, ModelPrompt, RerankResult, RetriverModel};
use rag_core::{Chunk, ServiceStatus};
use serde::Serialize;
use tokio::sync::{RwLock, mpsc::{UnboundedReceiver, UnboundedSender}};
use tracing::{debug, error, info, warn};
use utilites::Date;

use crate::{Error};

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
    reranker: RwLock<BgeReranker>,
    generator: RwLock<Generator>,
    qdrant_initialized: AtomicBool

}

impl Service
{
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self, Error>
    {
        let generator = Generator::load(Arc::clone(&emb_cfg))
            .inspect_err(|e| error!("Error when loading generator `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "generator".to_owned(), source: e })?;
        let retriver = RetriverModel::new(Arc::clone(&emb_cfg)).await
            .inspect_err(|e| error!("Error when loading retriver model `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "retriver".to_owned(), source: e })?;
        let reranker = BgeReranker::new(Arc::clone(&emb_cfg)).await
            .inspect_err(|e| error!("Error when loading reranker model `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "reranker".to_owned(), source: e })?;
        let slf = Self
        {
            embedding_config: emb_cfg,
            retriver: RwLock::new(retriver),
            reranker: RwLock::new(reranker),
            generator: RwLock::new(generator),
            qdrant_initialized: AtomicBool::new(false)
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
    fn qdrant_config(&self) -> QdrantConfig
    {
        QdrantConfig
        {
            url: "http://localhost:6334".to_owned(),
            collection_name: "law_collection".to_owned(),
            distance: database::Distance::Cosine
        }
    }

    pub async fn init_qdrant(&self) -> Result<(), Error>
    {
        if self.qdrant_initialized.load(std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(());
        }
        let cfg = self.qdrant_config();
        let _ = self.check_load_embedding_models().await?;
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        let qm = QdrantManager::new(cfg, &retriver, &reranker).await?;
        qm.ensure_collection().await?;
        Ok(())
    }
    //tokio spawn inside, may be long operation
    //progress messages and errors sends via sender
    pub async fn add_chunks_to_qdrant(self: Arc<Self>, chunks: Vec<Chunk>) -> Result<UnboundedReceiver<ServiceStatus>, Error>
    {
        let cfg = self.qdrant_config();
        let drop_service = self;
        let service = Arc::clone(&drop_service);
        let _ = service.check_load_embedding_models().await?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move 
            {
                let reranker = service.reranker.read().await;
                let retriver = service.retriver.read().await;
                
                let qm = QdrantManager::new(cfg, &retriver, &reranker).await
                    .inspect_err(|e| error!("Error creating QdrantManager: {}", e));
                if let Ok(qm) = qm
                {
                    //auto send error to client
                    let _added = qm.add_chunks_to_qdrant(chunks, sender).await;
                }
                else
                {
                    let _ = sender.send(ServiceStatus::qdrant_error(qm.err().unwrap()));
                }
            }
        );
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
    pub async fn search(&self, query: &str, limit: usize, reranker_limit: usize) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        let cfg = self.qdrant_config();
        let _ = self.check_load_embedding_models().await?;
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        let qm = QdrantManager::new(cfg, &retriver, &reranker).await
            .inspect_err(|e| error!("Error creating QdrantManager: {}", e))?;
        let result = qm.semantic_search(&query, limit, reranker_limit, None).await?;
        Ok(result)
        
       
       
        // let result = tokio::spawn(async move 
        //     {
        //         let reranker = service.reranker.read().await;
        //         let retriver = service.retriver.read().await;
        //         let ret = retriver.as_ref().unwrap();
        //         let rer = reranker.as_ref().unwrap();
        //         let qm = QdrantManager::new(cfg, ret, rer).await
        //             .inspect_err(|e| error!("Error creating QdrantManager: {}", e));
        //         if let Ok(qm) = qm
        //         {
        //             let result = qm.semantic_search(&query, limit, reranker_limit, None).await?;
        //             Ok(result)
        //         }
        //         else
        //         {
        //             Err(qm.err().unwrap())
        //         } 
        //     }
        // );
        // let result = result.await?
        //     .map_err(|e| Error::DatabaseError(e));
        // result
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
        let _ = self.init_qdrant().await
            .inspect_err(|e| error!("Qdrant initialize error: {}", e))?;
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
        let _ = self.init_qdrant().await?;
        Ok(last_state)
    }

    pub async fn delete_document_from_qdrant(&self, hash: &str) -> Result<(), Error>
    {
        let _ = self.check_load_embedding_models().await?;
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        let qm = QdrantManager::new(self.qdrant_config(), &retriver, &reranker).await
            .inspect_err(|e| error!("Error creating QdrantManager: {}", e))?;
        let _ = qm.delete_document(hash).await?;
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
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let service = Arc::new(super::Service::new(emb_cfg).await.unwrap());
        service.load_embedding_models().await.unwrap();
        service.init_qdrant().await.unwrap();
        let query = "Как взыскивается задолженность с налогоплательщика?";
        let context = service.search(query, 5, 5).await.unwrap();
        service.unload_embedding_models().await.unwrap();
        service.load_generator_model().await.unwrap();
        let mut receiver = service.genereate_response(query, context).await.unwrap();
        info!("GEN RESPONSE:");
        while let Some(output) = receiver.recv().await
        {
            info!("{:?}", output);
        }
    }
}

use std::{collections::HashMap, sync::{Arc, atomic::AtomicBool}};
use database::{ChunkProcess, QdrantConfig, QdrantManager, SearchResult};
use embedding::{BgeReranker, Chunk, ChunkMeta, Chunker, EmbeddingConfiguration, Embeddings, Generator, ModelPrompt, RerankResult};
use tokio::sync::{RwLock, mpsc::{UnboundedReceiver, UnboundedSender}};
use tracing::{debug, error, info, warn};
use utilites::Date;

use crate::{Error, html_converter::HtmlConverter};

pub struct Service
{
    embedding_config: Arc<EmbeddingConfiguration>,
    pub retriver: RwLock<Option<Embeddings>>,
    reranker: RwLock<Option<BgeReranker>>,
    generator: RwLock<Option<Generator>>,
    generator_model_is_load: AtomicBool

}

impl Service
{
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self, Error>
    {
        let generator = Generator::load(Arc::clone(&emb_cfg))
            .inspect_err(|e| error!("Error when loading generator `{}`", e))
            .map_err(|e| Error::GeneratorModelLoadError { source: e })?;
        let slf = Self
        {
            embedding_config: emb_cfg,
            retriver: RwLock::new(None),
            reranker: RwLock::new(None),
            generator: RwLock::new(Some(generator)),
            generator_model_is_load: AtomicBool::new(false)
        };
       Ok(slf)
    }
    pub async fn load_generator_model(&self) -> Result<(), Error>
    {
        if self.generator_model_is_load.load(std::sync::atomic::Ordering::Relaxed)
        {
            warn!("Generator model already loaded");
            Ok(())
        }
        else 
        {
            let mut lock = self.generator.write().await;
            let generator = lock.take();
            if let Some(generator) = generator
            {
                let with_model = generator.load_model().await
                    .map_err(|e| Error::GeneratorModelLoadError { source: e })?;
                *lock = Some(with_model);
                self.generator_model_is_load.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Ok(())
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
        let cfg = self.qdrant_config();
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        if let (Some(ret), Some(rer)) = (reranker.as_ref(), retriver.as_ref())
        {
            let qm = QdrantManager::new(cfg, rer, ret).await?;
            qm.ensure_collection().await?;
            Ok(())
        }
        else
        {
            if reranker.is_none()
            {
                Err(Error::RerankerModelLoadError)
            }
            else 
            {
                Err(Error::RetriverModelLoadError)
            }
        }
    }
    //tokio spawn inside, may be long operation
    //progress messages and errors sends via sender
    pub async fn add_chunks_to_qdrant(self: Arc<Self>, chunks: Vec<Chunk>, sender: tokio::sync::mpsc::UnboundedSender<ChunkProcess>) -> Result<(), Error>
    {
        let cfg = self.qdrant_config();
        let drop_service = self;
        let service = Arc::clone(&drop_service);
        let reranker = drop_service.reranker.read().await;
        let retriver = drop_service.retriver.read().await;
        if reranker.is_none()
        {
            return Err(Error::RerankerModelLoadError);
        }
        if retriver.is_none() 
        {
            return Err(Error::RetriverModelLoadError);
        }
        drop(reranker);
        drop(retriver);
        tokio::spawn(async move 
            {
                let reranker = service.reranker.read().await;
                let retriver = service.retriver.read().await;
                if let (Some(ret), Some(rer)) = (reranker.as_ref(), retriver.as_ref())
                {
                    let qm = QdrantManager::new(cfg, rer, ret).await
                        .inspect_err(|e| error!("Error creating QdrantManager: {}", e));
                    if let Ok(qm) = qm
                    {
                        //auto send error to client
                        let _added = qm.add_chunks_to_qdrant(chunks, sender).await;
                    }
                    else
                    {
                        let _ = sender.send(ChunkProcess::qdrant_error(qm.err().unwrap()));
                    }
                }
            }
        );
        Ok(())
        
    }

    pub async fn search(&self, query: &str, limit: usize, reranker_limit: usize) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        let cfg = self.qdrant_config();
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        if reranker.is_none()
        {
            return Err(Error::RerankerModelLoadError);
        }
        if retriver.is_none() 
        {
            return Err(Error::RetriverModelLoadError);
        }
        let ret = retriver.as_ref().unwrap();
        let rer = reranker.as_ref().unwrap();
        let qm = QdrantManager::new(cfg, ret, rer).await
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

    pub async fn load_embedding_models(&self) -> Result<(), Error>
    {
        let mut lock_retriver = self.retriver.write().await;
        let mut lock_reranker = self.reranker.write().await;
        if lock_retriver.is_none()
        {
            let retriver = embedding::Embeddings::new(Arc::clone(&self.embedding_config)).await
                .map_err(|e| Error::EmbeddingModelLoadError { source: e })?;
            *lock_retriver = Some(retriver);
        }
        drop(lock_retriver);
        if lock_reranker.is_none()
        {
            let reranker = embedding::BgeReranker::new(Arc::clone(&self.embedding_config)).await
                .map_err(|e| Error::EmbeddingModelLoadError { source: e })?;
            *lock_reranker = Some(reranker);
        }
        Ok(())
    }
    pub async fn unload_generator_model(&self) -> Result<(), Error>
    {
        let mut lock = self.generator.write().await;
        let mut generator = lock.as_mut();
        if let Some(generator) = generator.as_mut()
        {
            generator.unload_model();
            self.generator_model_is_load.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }
    pub async fn unload_embedding_models(&self) -> Result<(), Error>
    {
        let mut lock = self.retriver.write().await;
        *lock = None;
        let mut lock = self.reranker.write().await;
        *lock = None;
        Ok(())
    }

    pub async fn load_all(&self) -> Result<(), Error>
    {
        let _generator = self.load_generator_model().await?;
        let _qdrant = self.load_embedding_models().await?;
        let _ = self.init_qdrant().await?;
        Ok(())
    }
    pub async fn get_chunks(&self, sign_date: Date, number: &str, sender: UnboundedSender<ChunkProcess>) -> Result<Vec<Chunk>, Error>
    {
        let model = self.retriver.read().await;
        if model.is_none()
        {
            return Err(Error::RetriverModelLoadError);
        }
        let model = model.as_ref().unwrap();
        let converter = HtmlConverter{};
        let result = 
            systema_client::SystemaClient::get_document(
                sign_date,
                number, converter).await?;  
        let hash = result.hash();
        let count = result.node_count();
        //если будем разбивать на буольшее количество чанков чем один абзац - один чанк то необходимо поправить
        let mut chunks = Vec::with_capacity(result.node_count());
        info!("Ноды документы были успешно получены: {} шт.", result.node_count());
        let chunker = Chunker::new(&model.model()).await.unwrap();
        let mut current = 1;
        for node in &result
        {
            //бьем текст на куски тут и для каждого создаем чанку
            let splitted = chunker.split_text(node.converted_content()).await.unwrap();
            for text in splitted
            {
                let chunk = Chunk
                {
                    publication_url: result.publication_url().to_owned(),
                    document_url: format!("http://actual.pravo.gov.ru/list.html#hash={}", result.hash()),
                    title: result.title().to_owned(),
                    number: result.number().to_owned(),
                    sign_date: result.sign_date().to_owned(),
                    hash: result.hash().to_owned(),
                    path: result.find_all_parents_as_str(&node),
                    links_hashes: node.links_hashes().cloned(),
                    content: text.content,
                    embeddings: None,
                    meta: Some(ChunkMeta
                    {
                        chunk_index: text.chunk_index,
                        token_count: text.token_count
                    })
                };
                let _ = sender.send(ChunkProcess::process_chunk(hash, current, count));
                chunks.push(chunk);
                current +=1;
            }
        }
        Ok(chunks)
    }


    pub async fn genereate_response(self: Arc<Self>, query: &str, context: Vec<RerankResult<SearchResult>>) -> Result<UnboundedReceiver<String>, Error>
    {
        let service = self;
        
        let generator = service.generator.read().await;
        if generator.is_none()
        {
            return Err(Error::GeneratorModelNotLoadError);
        }
        drop(generator);
        let task_service = Arc::clone(&service);
        let query = query.to_owned();
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        std::thread::spawn(async move ||
        {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut generator_lock = task_service.generator.write().await;
                let generator = generator_lock.as_mut().unwrap();
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

    use crate::{chunks, logger};

     #[tokio::test]
    async fn test_service()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let service = Arc::new(super::Service::new(emb_cfg).await.unwrap());
        service.load_embedding_models().await.unwrap();
        service.init_qdrant().await.unwrap();
        let date = Date::new_date(31, 07, 2025);
        let number = "287-ФЗ";  
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(output) = receiver.recv().await
            {
                info!("{:?}", output);
            }
        });
        let chunks = service.get_chunks(date, number,sender).await.unwrap();
        let add_qd_clone = Arc::clone(&service);
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(output) = receiver.recv().await
            {
                info!("{:?}", output);
            }
        });
        let _ = add_qd_clone.add_chunks_to_qdrant(chunks, sender).await.unwrap();
        let query = "Как взыскивается задолженность с налогоплательщика?";
        let context = service.search(query, 5, 5).await.unwrap();
        service.unload_embedding_models().await.unwrap();
        let mut receiver = service.genereate_response(query, context).await.unwrap();
        info!("GEN RESPONSE:");
        while let Some(output) = receiver.recv().await
        {
            info!("{:?}", output);
        }
        
    }
}

use std::{collections::HashMap, sync::{Arc, atomic::AtomicBool}};

use database::{ChunkProcess, QdrantConfig, QdrantManager, SearchResult};
use embedding::{BgeReranker, EmbeddingConfiguration, Generator, RerankResult};
use logger::warn;
use tokio::sync::RwLock;
use rag_service::{Chunk, Embeddings};
use tracing::error;
use utilites::Date;

use crate::{Error, types::Document};

pub struct EmbeddingService
{
    embedding_config: Arc<EmbeddingConfiguration>,
    pub retriver: RwLock<Option<Embeddings>>,
    reranker: RwLock<Option<BgeReranker>>,
    generator: RwLock<Option<Generator>>,
    generator_model_is_load: AtomicBool

}

impl EmbeddingService
{
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self, Error>
    {
        let generator = Generator::load(Arc::clone(&emb_cfg))
            .inspect_err(|e| error!("Error when loading generator `{}`", e))?;
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
                let with_model = generator.load_model().await?;
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
            collection_name: "test_collection".to_owned(),
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
            Err(Error::ModelsError("не все модели были загружены!".to_owned()))
        }
    }

    pub async fn add_chunks_to_qdrant(&self, chunks: Vec<Chunk>, sender: tokio::sync::mpsc::UnboundedSender<ChunkProcess>) -> Result<(), Error>
    {
        let cfg = self.qdrant_config();
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        if let (Some(ret), Some(rer)) = (reranker.as_ref(), retriver.as_ref())
        {
            let qm = QdrantManager::new(cfg, rer, ret).await?;
            qm.add_chunks_to_qdrant(chunks, sender).await?;
            Ok(())
        }
        else
        {
            Err(Error::ModelsError("не все модели были загружены!".to_owned()))
        }
    }
    pub async fn search(&self, query: &str) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        let cfg = self.qdrant_config();
        let reranker = self.reranker.read().await;
        let retriver = self.retriver.read().await;
        if let (Some(ret), Some(rer)) = (reranker.as_ref(), retriver.as_ref())
        {
            let qm = QdrantManager::new(cfg, rer, ret).await?;
            let result = qm.semantic_search(query, 7, None).await?;
            Ok(result)
        }
        else
        {
            Err(Error::ModelsError("не все модели были загружены!".to_owned()))
        }
    }

    pub async fn load_embeddings(&self) -> Result<(), Error>
    {
        let mut lock_retriver = self.retriver.write().await;
        let mut lock_reranker = self.reranker.write().await;
        if lock_retriver.is_none()
        {
            let retriver = embedding::Embeddings::new(Arc::clone(&self.embedding_config)).await?;
            *lock_retriver = Some(retriver);
        }
        drop(lock_retriver);
        if lock_reranker.is_none()
        {
            let reranker = embedding::BgeReranker::new(Arc::clone(&self.embedding_config)).await?;
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
            generator.unload_model()?;
            self.generator_model_is_load.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn load_all(&self) -> Result<(), Error>
    {
        let _generator = self.load_generator_model().await?;
        let _qdrant = self.load_embeddings().await?;
        let _ = self.init_qdrant().await?;
        Ok(())
    }
}

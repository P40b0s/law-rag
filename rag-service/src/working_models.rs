use std::sync::Arc;

use embedding::{BgeReranker, Generator, GeneratorLlama1b, Model, RetriverModel};
use rag_core::{Embedder, EmbeddingConfiguration, Encoder, Reranker};
use tokio::sync::RwLock;
use tracing::error;

use crate::Error;

type RetriverImpl = RetriverModel;
type GeneratorImpl = GeneratorLlama1b;
type RerankerImpl = BgeReranker;
pub struct WorkingModels
{
    retriver: RwLock<RetriverImpl>,
    reranker: RwLock<RerankerImpl>,
    generator: RwLock<GeneratorImpl>,
}
impl WorkingModels
{
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self, Error>
    {
        let generator = GeneratorImpl::load(Arc::clone(&emb_cfg))
            .inspect_err(|e| error!("Error when loading generator `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "generator".to_owned(), source: e })?;
        let retriver = RetriverImpl::new(Arc::clone(&emb_cfg)).await
            .inspect_err(|e| error!("Error when loading retriver model `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "retriver".to_owned(), source: e })?;
        let reranker = RerankerImpl::new(Arc::clone(&emb_cfg)).await
            .inspect_err(|e| error!("Error when loading reranker model `{}`", e))
            .map_err(|e| Error::ModelLoadError { model: "reranker".to_owned(), source: e })?;
        Ok(Self
        {
            generator: RwLock::new(generator),
            retriver: RwLock::new(retriver),
            reranker: RwLock::new(reranker)
        })
    }
    pub async fn get_encoder(&self) -> tokio::sync::RwLockReadGuard<'_, impl Encoder>
    {
        self.retriver.read().await
    }
    pub async fn get_retriver(&self) -> tokio::sync::RwLockReadGuard<'_, impl Model + Embedder>
    {
        self.retriver.read().await
    }
    pub async fn get_retriver_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, impl Model + Embedder>
    {
        self.retriver.write().await
    }
    pub async fn get_reranker(&self) -> tokio::sync::RwLockReadGuard<'_, impl Model + Reranker>
    {
        self.reranker.read().await
    }
    pub async fn get_reranker_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, impl Model + Reranker>
    {
        self.reranker.write().await
    }
    pub async fn get_generator(&self) -> tokio::sync::RwLockReadGuard<'_, impl Generator>
    {
        self.generator.read().await
    }
    pub async fn get_generator_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, impl Generator>
    {
        self.generator.write().await
    }

}
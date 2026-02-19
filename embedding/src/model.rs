use std::{ops::Deref, sync::Arc};
use candle_core::{Device, Tensor};
use candle_transformers::models::{bert::{BertModel, Config}};
use rag_core::EmbeddingConfiguration;
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use anyhow::Result;

pub trait Model
where  Self: Sized
{
    fn name(&self) -> &ModelName;
    fn tokenizer(&self) -> &Tokenizer;
    fn tokenizer_mut(&mut self) -> &mut Tokenizer;
    fn embedding_config(&self) -> Arc<EmbeddingConfiguration>;
    fn device(&self) -> &Device;
    fn load_model(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
    //fn model(&self) -> Result<&M>;
    fn forward(
        &self,
        input_ids: &Tensor,
        attention_mask: &Tensor,
        token_type_ids: &Tensor,
    ) -> Result<Tensor>;
    fn model_is_loaded(&self) -> bool;
    fn unload_model(&mut self);
    ///размерность выходного вектора эмбеддинга
    fn dimension(&self) -> usize;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModelName
{
    M3,
    RerankerM3
}

impl Deref for ModelName
{
    type Target = str;
    fn deref(&self) -> &Self::Target 
    {
        match self
        {
            ModelName::M3 => "BAAI/bge-m3",
            ModelName::RerankerM3 => "BAAI/bge-reranker-v2-m3"
        }
    }
}

impl AsRef<str> for ModelName
{
    fn as_ref(&self) -> &str 
    {
        &self
    }
}
use std::{ops::Deref, sync::Arc};
use candle_core::Device;
use candle_transformers::models::{bert::{BertModel, Config}};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use anyhow::Result;

use crate::EmbeddingConfiguration;

pub trait Model
where  Self: Sized
{
    fn name(&self) -> &ModelName;
    fn tokenizer(&self) -> &Tokenizer;
    fn tokenizer_mut(&mut self) -> &mut Tokenizer;
    fn config(&self) -> &Config;
    fn embedding_config(&self) -> Arc<EmbeddingConfiguration>;
    fn device(&self) -> &Device;
    fn load_model(self) -> impl std::future::Future<Output = Result<Self>> + Send;
    fn model(&self) -> Result<&BertModel>;
    fn unload_model(self) -> Result<()>;
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
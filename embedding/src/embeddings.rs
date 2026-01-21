use std::sync::Arc;

use candle_transformers::models::bert::{BertModel, Config, HiddenAct, DTYPE};
use candle_core::Tensor;
use candle_nn::VarBuilder;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::{PaddingParams, Tokenizer};
use tracing::{debug, info};
use crate::{EmbeddingConfiguration, error::{Error, Result}, model::Model, retriver::RetriverModel};

pub struct Embeddings
{
    context_model: RetriverModel
}
impl Embeddings
{
    pub fn dimension(&self) -> usize
    {
        self.context_model.dimension
    }
    pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self>
    {
        let model = RetriverModel::new(emb_cfg).await?;
        let model = model.load_model().await?;
        Ok(Self
        {
            context_model: model
        })
    }
    pub fn model(&self) -> &RetriverModel
    {
        &self.context_model
    }
    async fn embed_vec(&self, text: &str) -> Result<Vec<f32>>
    {
        let embeddings = self.embed_tensor(text).await?;
        Ok(embeddings.to_vec1()?)
    }

    pub async fn embed_vec_batch(&self, texts: &[&str]) -> Result<Vec<f32>>
    {
        let embeddings = self.embed_tensor_batch(texts).await?;
        Ok(embeddings.to_vec1()?)
    }
    pub async fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<f32>>
    {
        let tensor = self.embed_tensor_batch(texts).await?;
        let mut vector = tensor.to_vec2()?;
        debug!("emb vector len: {}", vector.len());
        if vector.len() > 1
        {
            return Err(Error::VectorError);
        }
        let first = vector.pop();
        if let Some(first) = first
        {
            Ok(first)
        }
        else 
        {
            Err(Error::VectorError)
        }
    }
    pub async fn embed_tensor_batch(&self, texts: &[&str]) -> Result<Tensor>
    {
        let start = std::time::Instant::now();
        let device = self.context_model.device();
        let model = self.context_model.model()?;
         let tokens = self.context_model.tokenizer()
            .encode_batch(texts.to_vec(), true)?;

        let token_ids = tokens
            .iter()
            .map(|tokens| 
            {
                let tokens = tokens.get_ids().to_vec();
                Ok(Tensor::new(tokens.as_slice(), device)?)
            })
            .collect::<Result<Vec<_>>>()?;

        let attention_mask = tokens
            .iter()
            .map(|tokens| 
            {
                let tokens = tokens.get_attention_mask().to_vec();
                Ok(Tensor::new(tokens.as_slice(), device)?)
            })
            .collect::<Result<Vec<_>>>()?;

        let token_ids = Tensor::stack(&token_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_mask, 0)?;
        let token_type_ids = token_ids.zeros_like()?;
        info!("running inference {:?}", token_ids.shape());
        let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        info!("generated embeddings {:?}", embeddings.shape());
        let embeddings = Self::mean_pooling(&embeddings, &attention_mask)?;
        info!("pooled embeddings {:?} in {:?} s", embeddings.shape(), start.elapsed());
        Ok(embeddings)
    }

    fn mean_pooling(embeddings: &Tensor, attention_mask: &Tensor) -> Result<Tensor> 
    {
        let attention_mask_for_pooling = attention_mask.to_dtype(DTYPE)?.unsqueeze(2)?;
        let sum_mask = attention_mask_for_pooling.sum(1)?;
        let embeddings = (embeddings.broadcast_mul(&attention_mask_for_pooling)?).sum(1)?;
        let embeddings = embeddings.broadcast_div(&sum_mask)?;
        let embeddings = embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?;
        Ok(embeddings)
    }

    async fn embed_tensor(&self, text: &str) -> Result<Tensor>
    {
        let start = std::time::Instant::now();
        let model = self.context_model.model()?;
        let tokens = self.context_model.tokenizer()
            .encode(text, true)?;

        let token_ids = Tensor::new(tokens.get_ids(), self.context_model.device())?.unsqueeze(0)?;
        let attention_mask = Tensor::new(tokens.get_attention_mask(), self.context_model.device())?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;
        info!("running inference {:?}", token_ids.shape());
        let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        info!("generated embeddings {:?}", embeddings.shape());
        let embeddings = Self::mean_pooling(&embeddings, &attention_mask)?;
        info!("pooled embeddings {:?} in {:?} s", embeddings.shape(), start.elapsed());
        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use tracing::info;

    use crate::{EmbeddingConfiguration, logger};

    #[tokio::test]
    async fn test_embed()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let emb = super::Embeddings::new(emb_cfg).await.unwrap();
        let text = &["Тестовый текст лалалал"];
        let e =  emb.embed_tensor_batch(text).await;
        info!("{:?}", e);
    }
    #[tokio::test]
    async fn test_embed2()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let emb = super::Embeddings::new(emb_cfg).await.unwrap();
        let text = "Тестовый текст лалалал";
        let e =  emb.embed_tensor(text).await;
        info!("{:?}", e);
    }
}
use std::{sync::Arc};
use candle_core::{Device, Tensor};
use rag_core::{EmbeddingConfiguration, ModelName};
use tokenizers::Tokenizer;
use anyhow::Result;

pub trait Model
where  Self: Sized
{
    fn name(&self) -> &ModelName;
    fn tokenizer(&self) -> &Tokenizer;
    fn tokenizer_mut(&mut self) -> &mut Tokenizer;
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
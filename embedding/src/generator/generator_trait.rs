use std::sync::Arc;

use anyhow::Result;
use candle_transformers::generation::LogitsProcessor;
use crate::{EmbeddingConfiguration, token_output_stream::TokenOutputStream};

/// Common settings for all generator models
#[derive(Debug, Clone)]
pub struct GeneratorSettings 
{
    /// The length of the sample to generate (in tokens).
    pub sample_len: usize,

    /// The temperature used to generate samples, use 0 for greedy sampling.
    pub temperature: f64,

    /// Nucleus sampling probability cutoff.
    pub top_p: Option<f64>,

    /// Only sample among the top K samples.
    pub top_k: Option<usize>,

    /// The seed to use when generating random samples.
    pub seed: u64,

    /// Process prompt elements separately.
    pub split_prompt: bool,

    /// Run on CPU rather than GPU even if a GPU is available.
    pub cpu: bool,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    pub repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    pub repeat_last_n: usize,

    /// Context window size
    pub n_ctx: u32,
}

impl Default for GeneratorSettings 
{
    fn default() -> Self 
    {
        GeneratorSettings 
        {
            sample_len: 1000,
            temperature: 0.3,
            top_p: Some(0.9),
            top_k: Some(30),
            seed: 299792458,
            split_prompt: false,
            cpu: true,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            n_ctx: 8192,
        }
    }
}

/// Trait for text generation models
pub trait Generator: Send + Sync
where Self: Sized
{
    fn load(settings: Arc<EmbeddingConfiguration>) -> anyhow::Result<Self>;
    /// Load the model into memory
    fn load_model(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Unload the model from memory
    fn unload_model(&mut self);

    /// Check if model is loaded
    fn model_is_loaded(&self) -> bool;

    /// Get model size in bytes
    fn get_size_in_bytes(&self) -> usize;

    /// Get system prompt
    fn get_system_prompt(&self) -> &str;
    fn get_message<C: ToString + AsRef<str>>(&self, query: &str, context: &[C]) -> String;
    fn get_eof(&self) -> &'static str;

    /// Get token output stream
    fn get_token_stream(&self) -> TokenOutputStream;

    /// Get logits processor
    fn get_logits_processor(&self) -> LogitsProcessor;

    /// Generate response for query with context
    fn prompt<'a, C: ToString  + AsRef<str>>(
        &'a mut self,
        query: &'a str,
        context: &'a [C],
        sender: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<()>;
}

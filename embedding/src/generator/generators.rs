use std::sync::Arc;

use anyhow::anyhow;
use rag_core::{EmbeddingConfiguration, GeneratorConfig, ModelName};

use crate::{ExtendedGenerator, ExtendedGeneratorConfig, Generator, GeneratorLlama1b, GeneratorLlama8b};

pub enum Generators
{
    Llama1b(GeneratorLlama1b),
    Llama8b(GeneratorLlama8b),
    Extended(ExtendedGenerator),
}

impl Generators
{
    pub async fn load(cfg: Arc<EmbeddingConfiguration>, model_name: ModelName) -> anyhow::Result<Self>
    {
        match model_name
        {
            ModelName::Llama1b => Ok(Self::Llama1b(GeneratorLlama1b::load(cfg).await?)),
            ModelName::Llama8b => Ok(Self::Llama8b(GeneratorLlama8b::load(cfg).await?)),
            ModelName::Qwen32b => 
            {
                if let Some(url) = cfg.extended_generator_url.as_ref()
                {
                    let config = ExtendedGeneratorConfig::new(model_name, url, cfg.generator_temperature);
                    Ok(Self::Extended(ExtendedGenerator::new(config)))
                }
                else
                {
                    Err(anyhow!("Не указан адрес сервера генератора"))
                }
               
            },
            _ => Err(anyhow!("Указанная модель {} не предназначена для генерации контента", model_name))
        }
    }
}

impl Generator for Generators
{
    fn model_name(&self) -> &ModelName 
    {
        match self
        {
            Generators::Llama1b(g) => g.model_name(),
            Generators::Llama8b(g) => g.model_name(),
            Generators::Extended(g) => g.model_name(),
        }
    }
    fn load_model(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send 
    {
        async move 
        {
            match self 
            {
                Self::Llama1b(g) => g.load_model().await,
                Self::Llama8b(g) => g.load_model().await,
                Self::Extended(g) => g.load_model().await,
            }
        }
    }

    fn unload_model(&mut self) 
    {
        match self 
        {
            Self::Llama1b(g) => g.unload_model(),
            Self::Llama8b(g) => g.unload_model(),
            Self::Extended(g) => g.unload_model(),
        }
    }

    fn model_is_loaded(&self) -> bool 
    {
        match self 
        {
            Self::Llama1b(g) => g.model_is_loaded(),
            Self::Llama8b(g) => g.model_is_loaded(),
            Self::Extended(g) => g.model_is_loaded(),
        }
    }

    fn get_size_in_bytes(&self) -> usize 
    {
        match self 
        {
            Self::Llama1b(g) => g.get_size_in_bytes(),
            Self::Llama8b(g) => g.get_size_in_bytes(),
            Self::Extended(g) => g.get_size_in_bytes(),
        }
    }

    fn get_system_prompt(&self) -> &str 
    {
        match self 
        {
            Self::Llama1b(g) => g.get_system_prompt(),
            Self::Llama8b(g) => g.get_system_prompt(),
            Self::Extended(g) => g.get_system_prompt(),
        }
    }

    fn prompt<'a, C: ToString  + AsRef<str>>(
        &'a mut self,
        query: &'a str,
        context: &'a [C],
        sender: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> anyhow::Result<()> 
    {
        match self 
        {
            Self::Llama1b(g) => g.prompt(query, context, sender),
            Self::Llama8b(g) => g.prompt(query, context, sender),
            Self::Extended(g) => g.prompt(query, context, sender),
        }
    }
}
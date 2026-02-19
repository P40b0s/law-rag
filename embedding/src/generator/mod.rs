// Re-export generator implementations and trait
mod generator_trait;
mod generator_llama8b;
mod generator_llama1b;
mod generators;

pub use generator_trait::{Generator, GeneratorSettings};
pub use generator_llama8b::GeneratorLlama8b;
pub use generator_llama1b::GeneratorLlama1b;
pub use generators::Generators;

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use rag_core::EmbeddingConfiguration;

    use crate::{Generator, GeneratorLlama1b, logger};
   
    #[tokio::test]
    async fn load_generator()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let mut generator = GeneratorLlama1b::load(emb_cfg).unwrap();
        generator.load_model().await.unwrap();
    }
}

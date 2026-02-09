mod logger;
mod html_converter;
mod chunk;
mod error;
mod retriver;
mod reranker;
mod configuration;
mod model;
mod generator;
mod token_output_stream;
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use scraper::Node;
use systema_client::Converter;
use tracing::info;
use utilites::Date;
use html_converter::HtmlConverter;

pub use chunk::{Chunk, ChunkMeta, Chunker, ChunkedText};
pub use reranker::{BgeReranker, RerankResult};
pub use retriver::RetriverModel;
pub use generator::{Generator, GeneratorSettings, ModelPrompt};
pub use error::Error;
pub use model::Model;
pub use configuration::EmbeddingConfiguration;

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use systema_client::{DocumentNode, DocumentNodes};
    use tokenizers::Tokenizer;
    use tracing::{debug, info};
    use utilites::Date;
    use crate::{Chunk, EmbeddingConfiguration, HtmlConverter, chunk::{ChunkMeta, Chunker}, retriver::RetriverModel, logger};

    #[tokio::test]
    async fn test_converter()
    {
        logger::init();
        let converter = HtmlConverter{};
        let result = 
            systema_client::SystemaClient::get_document(
                Date::new_date(31, 07, 2025),
                "287-ФЗ", converter).await.unwrap();  

        let mut chunks = Vec::with_capacity(result.node_count());
        info!("Ноды документы были успешно получены: {} шт.", result.node_count());
         let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let model = RetriverModel::new(emb_cfg).await.unwrap();
        let chunker = Chunker::new(&model).await.unwrap();
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
                chunks.push(chunk);
            }
        }
    }
}
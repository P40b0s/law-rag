use embedding::{Chunk, ChunkMeta, Chunker};
use tracing::info;
use utilites::Date;
use crate::error::{Error, Result};
use crate::html_converter::HtmlConverter;
use crate::logger;

pub async fn get_chunks(sign_date: Date, number: &str) -> Result<Vec<Chunk>>
{
    logger::init();
    let converter = HtmlConverter{};
    let result = 
        systema_client::SystemaClient::get_document(
            sign_date,
            number, converter).await?;  

    let mut chunks = Vec::with_capacity(result.node_count());
    info!("Ноды документы были успешно получены: {} шт.", result.node_count());
    let chunker = Chunker::new().await.unwrap();
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
    Ok(chunks)
}

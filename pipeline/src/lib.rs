mod html_converter;
mod logger;
use serde::{Deserialize, Serialize};
use scraper::Node;
use systema_client::Converter;
use tracing::info;
use utilites::Date;
use embedding::{Chunk, ChunkMeta, ChunkedText, Embeddings, Chunker};
use html_converter::HtmlConverter;


//TODO перенести всю логику запроса и его обработки сюда в том числе вычисление эмбеддингов 
#[cfg(test)]
mod tests
{
    use systema_client::{DocumentNode, DocumentNodes};
    use tracing::{debug, info};
    use utilites::Date;
    use crate::{Chunk, HtmlConverter, logger};
    use super::{ChunkMeta, Chunker};

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
                    liks_hashes: node.links_hashes().cloned(),
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
       
        for chunk in &chunks
        {
           
            debug!("chunk created: {:#?}", chunk);
        }
    }
}
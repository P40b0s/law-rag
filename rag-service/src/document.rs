use database::{DocumentCardDbo, DocumentDbo};
use embedding::{Chunk, ChunkMeta, Chunker, RetriverModel};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use utilites::Date;
use anyhow::Result;
use crate::{HtmlConverter, logger};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document
{
    pub document_uri: String,        // URI исходного документа
    pub document_hash: String,
    pub document_title: String, // Заголовок документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: Date, // Дата подписания документа
    pub has_embeddings: bool,
    pub chunks_count: u32,
    pub chunks: Vec<Chunk>,
    pub status: u8,
}

impl From<DocumentDbo> for Document
{
    fn from(value: DocumentDbo) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            chunks: value.chunks,
            status: value.status as u8
        }   
    }
}

impl From<DocumentCardDbo> for Document
{
    fn from(value: DocumentCardDbo) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            chunks: Vec::with_capacity(0),
            status: value.status as u8
        }      
    }
}

impl From<Vec<Chunk>> for Document
{
    fn from(value: Vec<Chunk>) -> Self 
    {
        let first_chunk = value.first().unwrap().clone();
        Self 
        { 
            document_uri: first_chunk.document_url,
            document_hash: first_chunk.hash,
            document_title: first_chunk.title,
            document_number: first_chunk.number,
            document_sign_date: first_chunk.sign_date,
            has_embeddings: false,
            chunks_count: value.len() as u32,
            chunks: value,
            status: database::DocumentStatus::NotLoaded.into(),
            
        }
    }
}

pub async fn load_document(number: &str, date: Date, model: &RetriverModel) -> Result<Vec<Chunk>>
{
    logger::init();
    let converter = HtmlConverter{};
    let result = 
        systema_client::SystemaClient::get_document(
            date,
            number, converter).await?;  

    let mut chunks = Vec::with_capacity(result.node_count());
    info!("Ноды документы были успешно получены: {} шт.", result.node_count());
    let chunker = Chunker::new(&model).await?;
    for node in &result
    {
        //бьем текст на куски тут и для каждого создаем чанку
        let splitted = chunker.split_text(node.converted_content()).await?;
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

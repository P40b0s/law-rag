use serde::{Deserialize, Serialize};
use utilites::Date;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk
{
    pub publication_url: String,
    pub document_url: String,
    pub title: String,
    pub number: String,
    pub sign_date: Date,
    pub hash: String,
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub links_hashes: Option<Vec<String>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub embeddings: Option<Vec<f32>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub meta: Option<ChunkMeta>
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ChunkMeta
{
    //http://actual.pravo.gov.ru/list.html#hash=3582241193d46b766ef7d5ae7f8d50577be6bf8a5b6e0d77784769fe1e9628b4
    pub chunk_index: usize,
    pub token_count: usize
}


pub struct ChunkedText
{
    pub content: String,
    pub token_count: usize,
    pub chunk_index: usize,
    pub total_chunks: usize,
}


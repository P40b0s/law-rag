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
}

pub struct ChunkedText
{
    pub content: String,
    pub token_count: usize,
    pub chunk_index: usize,
    pub total_chunks: usize,
}


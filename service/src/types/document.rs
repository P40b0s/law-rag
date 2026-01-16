use pipeline::Chunk;
use serde::Serialize;
use utilites::Date;

pub struct Document
{
    pub sign_date: Date,
    pub number: String,
    pub chunks: Vec<Chunk>,
    pub document_uri: String
}

impl From<Vec<Chunk>> for Document
{
    fn from(value: Vec<Chunk>) -> Self 
    {
        let first_chunk = value.first().unwrap();
        Self 
        { 
            sign_date: first_chunk.sign_date.clone(),
            number: first_chunk.number.clone(),
            document_uri: first_chunk.document_url.clone(),
            chunks: value,
            
        }
    }
}

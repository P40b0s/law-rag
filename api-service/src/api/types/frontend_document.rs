use rag_core::Chunk;
use serde::Serialize;
use utilites::Date;

#[derive(Debug, Serialize)]
pub struct FrontendDocument
{
    pub date: Date,
    pub number: String,
    pub first_chunk: Option<Chunk>,
    pub status: LoadStatus
}

impl From<&Chunk> for FrontendDocument
{
    fn from(value: &Chunk) -> Self 
    {
        Self 
        { 
            date: value.sign_date.clone(),
            number: value.number.clone(),
            first_chunk: Some(value.clone()),
            status: LoadStatus::Complete
        }
    }
}

#[derive(Debug, Serialize)]
pub enum LoadStatus
{
    NotFound,
    Timeout,
    Complete,
    Pending
}

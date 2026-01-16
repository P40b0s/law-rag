use std::collections::HashMap;

use pipeline::Chunk;
use utilites::Date;

use crate::{Error, types::Document};

pub struct DocumentsService
{
    pub processed_documents: HashMap<String, Document>,
    pub vectorized_documents: HashMap<String, Document>
}
impl Default for DocumentsService
{
    fn default() -> Self 
    {
        Self
        {
            processed_documents: HashMap::new(),
            vectorized_documents: HashMap::new()
        }
    }
}
impl DocumentsService
{
    pub async fn get_document(&mut self, date: Date, number: &str) -> Result<Document, Error>
    {
        let dot_date = date.format(utilites::DateFormat::DotDate);
        let chunks = pipeline::load_document(number, date,).await?;
        if let Some(_) = chunks.first()
        {
            Ok(chunks.into())
        }
        else 
        {
            Err(Error::empty_chunk(number, dot_date))    
        }
    }
}

use std::{collections::HashMap, sync::Arc};

use database::{DocumentDbo, DocumentsTable};
use rag_service::Chunk;
use tracing::error;
use utilites::Date;

use crate::{Error, services::embedding_service::EmbeddingService, types::Document};

pub struct DatabaseService
{
    pub database: DocumentsTable,
}


impl DatabaseService
{
    pub async fn new() -> Result<Self, Error>
    {
        let service = DocumentsTable::new_default().await?;
        Ok(Self 
        { 
            database: service
        })
    }
    ///добавляем док только если получены чанки
    pub async fn add_document(&self, document: &Document) -> Result<(), Error>
    {
        if let Some(first_chunk) = document.chunks.first()
        {
            let _ = self.database.create_document(
                &document.document_uri,
                &first_chunk.title,
                &document.number,
                &document.sign_date,
                database::DocumentStatus::Loaded,
                false,
                &(document.chunks.len() as u32),
                document.chunks.as_slice()
            ).await.inspect_err(|e| error!("{}", e))?;
            Ok(())
        }
        else 
        {
            Err(Error::ChunksIsEmpty(document.sign_date.format(utilites::DateFormat::DotDate), document.number.clone()))
        }
    }

    pub async fn is_embedded(&self, doc_uri: &str) -> Result<(), Error>
    {
       let _ = self.database.update_status(database::DocumentStatus::Embedded, doc_uri).await.inspect_err(|e| error!("{e}"))?;
       Ok(())
    }
    pub async fn delete(&self, doc_uri: &str) -> Result<(), Error>
    {
       let _ = self.database.delete_document(doc_uri).await.inspect_err(|e| error!("{e}"))?;
       Ok(())
    }
}

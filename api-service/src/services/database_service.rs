use std::{collections::HashMap, sync::Arc};

use database::{DocumentDbo, DocumentsTable};
use rag_service::{Chunk, Document};
use tracing::error;
use utilites::Date;

use crate::{Error};

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
                &first_chunk.hash,
                &first_chunk.title,
                &document.document_number,
                &document.document_sign_date,
                database::DocumentStatus::Loaded,
                false,
                &(document.chunks.len() as u32),
                document.chunks.as_slice()
            ).await.inspect_err(|e| error!("{}", e))?;
            Ok(())
        }
        else 
        {
            Err(Error::ChunksIsEmpty(document.document_sign_date.format(utilites::DateFormat::DotDate), document.document_number.clone()))
        }
    }
    pub async fn get_documents_cards(&self) -> Result<Vec<Document>, Error>
    {
        let docs = self.database.get_documents_without_chunks().await.inspect_err(|e| error!("{}", e))?;
        Ok(docs.into_iter().map(|d| d.into()).collect())
    }
    pub async fn get_document(&self, hash: &str) -> Result<Document, Error>
    {
        let doc = self.database.get_document_by_hash(hash).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(doc.into())
    }

    pub async fn set_is_embedded(&self, doc_hash: &str) -> Result<(), Error>
    {
       let _ = self.database.update_status(database::DocumentStatus::Embedded, doc_hash).await
        .inspect_err(|e| error!("{e}"))?;
       Ok(())
    }
    pub async fn delete(&self, doc_hash: &str) -> Result<(), Error>
    {
       let _ = self.database.delete_document(doc_hash).await
        .inspect_err(|e| error!("{e}"))?;
       Ok(())
    }
}

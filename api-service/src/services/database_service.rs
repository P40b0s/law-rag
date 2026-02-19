use std::{collections::HashMap, sync::Arc};

use database::{CollectionsTable, DocumentDbo, DocumentsTable};
use rag_core::DocumentStatus;
use rag_service::{Collection, Document};
use tracing::error;
use utilites::Date;

use crate::{Error};

pub struct DatabaseService
{
    documents: DocumentsTable,
    collections: CollectionsTable
}


impl DatabaseService
{
    pub async fn new() -> Result<Self, Error>
    {
        let db_name = "documents";
        let documents = DocumentsTable::new_default(db_name).await?;
        let collections = CollectionsTable::new_default(db_name).await?;
        Ok(Self 
        { 
            documents,
            collections
        })
    }
    ///добавляем док только если получены чанки
    pub async fn add_document(&self, document: &Document) -> Result<(), Error>
    {
        if let Some(first_chunk) = document.chunks.first()
        {
            let _ = self.documents.create_document(
                &document.document_uri,
                &first_chunk.hash,
                &first_chunk.title,
                &document.document_number,
                &document.document_sign_date,
                DocumentStatus::Loaded,
                &(document.chunks.len() as u32),
                document.collection_id,
                document.chunks.as_slice()
            ).await.inspect_err(|e| error!("{}", e))?;
            Ok(())
        }
        else 
        {
            Err(Error::ChunksIsEmpty(document.document_sign_date.format(utilites::DateFormat::DotDate), document.document_number.clone()))
        }
    }
    pub async fn get_documents_cards(&self, offset: u32, limit: u32) -> Result<Vec<Document>, Error>
    {
        let docs = self.documents.get_documents_paginated(limit, offset).await.inspect_err(|e| error!("{}", e))?;
        Ok(docs.into_iter().map(|d| d.into()).collect())
    }
    pub async fn get_document(&self, hash: &str) -> Result<Document, Error>
    {
        let doc = self.documents.get_document_by_hash(hash).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(doc.into())
    }

    pub async fn set_is_embedded(&self, doc_hash: &str) -> Result<(), Error>
    {
       let _ = self.documents.update_status(DocumentStatus::Embedded, doc_hash).await
        .inspect_err(|e| error!("{e}"))?;
       Ok(())
    }
    pub async fn delete_document(&self, doc_hash: &str) -> Result<(), Error>
    {
       let _ = self.documents.delete_document(doc_hash).await
        .inspect_err(|e| error!("{e}"))?;
       Ok(())
    }
    pub async fn get_documents_count(&self) -> Result<usize, Error>
    {
        let count = self.documents.get_documents_count().await
            .inspect_err(|e| error!("{}", e))?;
        Ok(count)
    }
    pub async fn get_collections(&self) -> Result<Vec<Collection>, Error>
    {
        let collections = self.collections.get_collections().await
            .inspect_err(|e| error!("{}", e))?;
        Ok(collections.into_iter().map(|c| c.into()).collect())
    }
    pub async fn get_collection_by_name(&self, name: &str) -> Result<Collection, Error>
    {
        let collection = self.collections.get_collection_by_name(name).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(collection.into())
    }

    pub async fn update_collection(&self, id: uuid::Uuid, description: &str) -> Result<(), Error>
    {
        let _ = self.collections.update(description, id).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(())
    }

    pub async fn delete_collection(&self, id: uuid::Uuid) -> Result<(), Error>
    {
        let _ = self.collections.delete_collection(id).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(())
    }
    pub async fn get_collection_by_id(&self, id: uuid::Uuid) -> Result<Collection, Error>
    {
        let collection = self.collections.get_collection_by_id(id).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(collection.into())
    }
    pub async fn add_collection(&self, name: &str, description: &str) -> Result<Collection, Error>
    {
        let collection = self.collections.create_collection(name, description).await
            .inspect_err(|e| error!("{}", e))?;
        Ok(collection.into())
    }
}

use std::{collections::HashMap, sync::{Arc, atomic::AtomicBool}};

use rag_service::Chunk;
use tracing::error;
use utilites::Date;

use crate::{Error, services::{SSEService, database_service::DatabaseService, embedding_service::EmbeddingService}, types::Document};

pub struct DocumentsService
{
    pub embedding_service: Arc<EmbeddingService>,
    pub database_service: Arc<DatabaseService>,
    pub sse_service: Arc<SSEService>,
    pub embedding_in_progress: AtomicBool
}

impl DocumentsService
{
    pub fn new(emb_service: Arc<EmbeddingService>, database_service: Arc<DatabaseService>, sse_service: Arc<SSEService>) -> Self
    {
        Self 
        { 
            database_service, 
            embedding_service: emb_service,
            sse_service,
            embedding_in_progress: AtomicBool::new(false)
        }
    }
    pub async fn load_document(&mut self, date: Date, number: &str) -> Result<Document, Error>
    {
        let dot_date = date.format(utilites::DateFormat::DotDate);
        let emb_model = self.embedding_service.retriver.read().await;
        if let Some(ret) = emb_model.as_ref()
        {
            let chunks = rag_service::load_document(number, date, ret.model()).await?;
            if let Some(_) = chunks.first()
            {
                let document = chunks.into();
                let _ = self.database_service.add_document(&document).await?;
                Ok(document)
            }
            else 
            {
                Err(Error::ChunksIsEmpty(dot_date, number.to_owned()))
            }
        }
        else 
        {
            Err(Error::ModelsError("Retriver model is not loaded".to_owned()))    
        }
    }
    pub async fn embedding_document(&self, document: Document) -> Result<(), Error>
    {
        if self.embedding_in_progress.load(std::sync::atomic::Ordering::Release)
        {
            return Err(Error::EmbeddingInProgress(document.document_uri));
        }
        self.embedding_in_progress.store(true, std::sync::atomic::Ordering::Acquire);
        let chunks = document.chunks.clone();
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let sse_service = self.sse_service.clone();
        tokio::spawn(
            async move 
            {
                while let Some(msg) = receiver.recv().await
                {
                    sse_service.load_chunk_process(msg);
                }
            }
        );
        let emb_service = self.embedding_service.clone();
        let task = tokio::spawn(
            async move 
            {
                let _added = emb_service.add_chunks_to_qdrant(chunks, sender).await
                .inspect_err(|e| error!("{}", e));
            }
        );
        let _ = task.await;
        self.embedding_in_progress.store(false, std::sync::atomic::Ordering::Acquire);
        Ok(())
          
    }
}

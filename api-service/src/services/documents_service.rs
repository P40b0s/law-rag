use std::{collections::HashMap, sync::{Arc, atomic::AtomicBool}};

use database::SearchResult;
use embedding::RerankResult;
use rag_service::{Chunk, Document};
use tokio::sync::mpsc::unbounded_channel;
use tracing::error;
use utilites::Date;
use rag_service::Service;

use crate::{Error, services::{SSEService, database_service::DatabaseService}};

pub struct DocumentsService
{
    pub rag_service: Arc<Service>,
    pub database_service: Arc<DatabaseService>,
    pub sse_service: Arc<SSEService>,
    pub embedding_in_progress: Arc<AtomicBool>
}

impl DocumentsService
{
    pub fn new(rag_service: Arc<Service>, database_service: Arc<DatabaseService>, sse_service: Arc<SSEService>) -> Self
    {
        Self 
        { 
            database_service, 
            rag_service: rag_service,
            sse_service,
            embedding_in_progress: Arc::new(AtomicBool::new(false))
        }
    }
    pub async fn get_document(&self, date: Date, number: &str) -> Result<Document, Error>
    {
        let (sender, mut receiver) = unbounded_channel();
        let sse_service = self.sse_service.clone();
        let chunks = self.rag_service.get_chunks(date, number, sender).await?;
        while let Some(msg) = receiver.recv().await
        {
            sse_service.process(msg);
        }
        Ok(chunks.into())
    }

    ///Создание эмбеддингов и сохранение в бд qdrant, возвращаем управление сразу, рельтат передается через sse service
    pub async fn embedding_document(&self, document: Document) -> Result<(), Error>
    {
        if self.embedding_in_progress.load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(Error::EmbeddingInProgress(document.document_hash));
        }
        let rag_service = self.rag_service.clone();
        let sse_service = self.sse_service.clone();
        let process_flag = self.embedding_in_progress.clone();
        let mut receiver = rag_service.add_chunks_to_qdrant(document.chunks).await?;
        tokio::task::spawn(async move {
            while let Some(msg) = receiver.recv().await
            {
                sse_service.process(msg);
            }
            process_flag.store(false, std::sync::atomic::Ordering::Release);
        });
        Ok(())
    }
    ///ждем результата
    pub async fn search_context(&self, query: &str, limit: usize, rerenker_limit: usize) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        self.sse_service.message(format!("Идет процесс поиска контекста по запросу `{}`, это может занять несколько минут", query));
        let context = self.rag_service.search(query, limit, rerenker_limit).await?;
        Ok(context)
    }
    ///возвращаем управление сразу, клиент получает данные по sse service
    pub async fn generate_result(&self, query: &str, context: Vec<RerankResult<SearchResult>>) -> Result<(), Error>
    {
        let rag_service = self.rag_service.clone();
        let sse_service = self.sse_service.clone();
        sse_service.message(format!("Идет процесс генерации ответа на вопрос `{}`, из контекста, это может занять несколько десятков минут", query));
        let mut receiver = rag_service.genereate_response(query, context).await?;
        tokio::task::spawn(async move {
            while let Some(msg) = receiver.recv().await
            {
                sse_service.generator_message(msg);
            }
        });
        Ok(())
    }

}

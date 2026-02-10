use std::{collections::HashMap, sync::{Arc, atomic::AtomicBool}};

use database::{DocumentCardDbo, SearchResult};
use embedding::RerankResult;
use rag_core::{ProcessStatus, ServiceStatus};
use rag_service::{Document, DocumentCard, ModelsState};
use systema_client::SystemaClient;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{debug, error};
use utilites::Date;
use chunks_preparation::{SystemaActualConverter, SystemaActualChunker, Chunker};
use rag_service::Service;

use crate::{Error, services::{SSEService, database_service::DatabaseService}};

pub struct DocumentsService
{
    pub rag_service: Arc<Service>,
    pub database_service: Arc<DatabaseService>,
    pub sse_service: Arc<SSEService>,
    pub systema_client: Arc<SystemaClient>,
    pub embedding_in_progress: Arc<AtomicBool>
}

impl DocumentsService
{
    pub fn new(rag_service: Arc<Service>, database_service: Arc<DatabaseService>, sse_service: Arc<SSEService>, systema_client: Arc<SystemaClient>) -> Self
    {
        Self 
        { 
            database_service, 
            rag_service: rag_service,
            sse_service,
            systema_client,
            embedding_in_progress: Arc::new(AtomicBool::new(false))
        }
    }
    //получаем документ от строннего апи и сразу чанкуем его
    pub async fn get_document(&self, date: Date, number: &str) -> Result<Document, Error>
    {
        let date_str = date.format(utilites::DateFormat::DotDate);
        let converter = SystemaActualConverter{};
        let result = 
            systema_client::SystemaClient::get_document(
                date,
                number, converter).await?;
        let (sender, mut receiver) = unbounded_channel();
        let sse_service = self.sse_service.clone();
        tokio::spawn(async move 
        {
            while let Some(msg) = receiver.recv().await
            {
                sse_service.status_message(msg);
            }
        });
        
        let chunker = SystemaActualChunker::new(result);
        let retriver_lock = self.rag_service.get_retriver().await;
        let chunks = chunker.get_chunks(&*retriver_lock, sender).await?;
        debug!("Получили {} чанков для документа {} от {}", chunks.len(), number, date_str);
        Ok(chunks.into())
    }

    //получаем документ от строннего апи и сразу чанкуем его и добавляем в sqlite
    pub async fn get_document_and_add_to_db(&self, date: Date, number: &str) -> Result<DocumentCard, Error>
    {
        let document = self.get_document(date, number).await?;
        let _ = self.database_service.add_document(&document).await?;
        let mut card: DocumentCard = document.into();
        card.status = rag_core::DocumentStatus::Loaded;
        Ok(card)
    }
    ///documents directly from pravo.gov.ru
    pub async fn get_documents_list(&self, offset: u32, limit: u32) -> Result<Vec<Document>, Error>
    {
        let documents = self.database_service.get_documents_cards(offset, limit).await?;
        Ok(documents)
    }
    pub async fn delete_document(&self, hash: &str) -> Result<(), Error>
    {
        let _ = self.rag_service.delete_document_from_qdrant(hash).await?;
        let _ = self.database_service.delete(hash).await?;
        Ok(())
    }

    pub async fn load_generator_model(&self) -> Result<ModelsState, Error>
    {
        let state = self.rag_service.load_generator_model().await?;
        Ok(state)
    }
    pub async fn unload_generator_model(&self) -> Result<ModelsState, Error>
    {
        let state = self.rag_service.unload_generator_model().await?;
        Ok(state)
    }
    pub async fn load_embedding_model(&self) -> Result<ModelsState, Error>
    {
        let state = self.rag_service.load_embedding_models().await?;
        Ok(state)
    }
    pub async fn unload_embedding_model(&self) -> Result<ModelsState, Error>
    {
        let state = self.rag_service.unload_embedding_models().await?;
        Ok(state)
    }
    pub async fn embedding_document_from_sqlite(&self, hash: &str) -> Result<(), Error>
    {
        let document = self.database_service.get_document(hash).await?;
        if self.embedding_in_progress.load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(Error::EmbeddingInProgress(document.document_hash));
        }
        let rag_service = self.rag_service.clone();
        let sse_service = self.sse_service.clone();
        let process_flag = self.embedding_in_progress.clone();
        let db_service = self.database_service.clone();
        let mut receiver = rag_service.add_chunks_to_qdrant(document.chunks).await?;
        let hash = hash.to_owned();
        tokio::task::spawn(async move {
            let mut has_error = false;
            while let Some(msg) = receiver.recv().await
            {
                if let ProcessStatus::Error = msg.status
                {
                    has_error = true;
                }
                sse_service.status_message(msg);
            }
            if !has_error
            {
                let _ = db_service.set_is_embedded(&hash).await;
            }
            process_flag.store(false, std::sync::atomic::Ordering::Release);
        });
        Ok(())
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
        tokio::task::spawn(async move 
        {
            while let Some(msg) = receiver.recv().await
            {
                sse_service.status_message(msg);
            }
            process_flag.store(false, std::sync::atomic::Ordering::Release);
        });
        Ok(())
    }
    ///ждем результата
    pub async fn search_context(&self, query: &str, limit: usize, reranker_limit: usize) -> Result<Vec<RerankResult<SearchResult>>, Error>
    {
        let msg = ServiceStatus::mesage(format!("Идет процесс поиска контекста по запросу `{}`, это может занять несколько минут", query));
        self.sse_service.status_message(msg);
        let context = self.rag_service.search(query, limit, reranker_limit).await?;
        Ok(context)
    }
    ///возвращаем управление сразу, клиент получает данные по sse service
    pub async fn generate_result(&self, query: &str, context: Vec<RerankResult<SearchResult>>) -> Result<(), Error>
    {
        let rag_service = self.rag_service.clone();
        let sse_service = self.sse_service.clone();
        let msg = ServiceStatus::mesage(format!("Идет процесс генерации ответа на вопрос `{}`, из контекста, это может занять несколько десятков минут", query));
        sse_service.status_message(msg);
        let mut receiver = rag_service.genereate_response(query, context).await?;
        tokio::task::spawn(async move {
            while let Some(msg) = receiver.recv().await
            {
                let msg = ServiceStatus::mesage(msg);
                sse_service.status_message(msg);
            }
        });
        Ok(())
    }

}

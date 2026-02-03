use std::sync::Arc;
use embedding::EmbeddingConfiguration;
use logger::error;
use rag_service::Service;
use uuid::Uuid;

use crate::{configuration::Configuration, services::{DatabaseService, DocumentsService, SSEService}};

pub struct Services
{
    pub sse_service: Arc<SSEService>,
    pub documents_service: Arc<DocumentsService>,
    pub database_service: Arc<DatabaseService>,
    pub rag_service: Arc<rag_service::Service>
}

pub struct AppState
{
    pub services: Services,
    pub configuration: Arc<Configuration>
}

impl AppState
{
    //TODO пробросить все конфигурации через главную
    pub async fn initialize() -> Result<AppState, crate::Error>
    {
        let configuration = Arc::new(Configuration::load());
        let sse_service = Arc::new(SSEService::new());
        //FIXME убрать
        let emb_cfg = Arc::new( EmbeddingConfiguration::default());
        let rag_service = Arc::new(Service::new(emb_cfg).await?);
        let database_service = Arc::new(DatabaseService::new().await?);
        let documents_service = Arc::new(DocumentsService::new(rag_service.clone(), database_service.clone(), sse_service.clone()));
        let services = Services
        {
            documents_service,
            database_service,
            rag_service,
            sse_service,
        };
        Ok(Self
        {
            services,
            configuration
        })
    }
    pub fn get_services(&self) -> &Services
    {
        &self.services
    }
}
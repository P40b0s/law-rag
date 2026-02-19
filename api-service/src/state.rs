use std::sync::Arc;
use systema_client::SystemaClient;
use tracing::error;
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
    pub async fn initialize() -> Result<AppState, crate::Error>
    {
        let configuration = Arc::new(Configuration::new()?);
        let sse_service = Arc::new(SSEService::new());
        let rag_service = Arc::new(Service::new(configuration.embedding_configuration.clone(), configuration.qdrant_configuration.clone()).await?);
        let database_service = Arc::new(DatabaseService::new().await?);
        let systema_client = Arc::new(SystemaClient{});
        let documents_service = Arc::new(DocumentsService::new(rag_service.clone(), database_service.clone(), sse_service.clone(), systema_client));
        let services = Services
        {
            documents_service,
            database_service,
            rag_service,
            sse_service,
        };
        // tokio::spawn(
        //     async move{
        //         loop {
        //             tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        //             sse_service.status_message(ServiceStatus::mesage("message 1".to_owned()));
        //             tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        //             sse_service.status_message(ServiceStatus::mesage("amessage 2".to_owned()));
        //         }
               
        //     }
        // );
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
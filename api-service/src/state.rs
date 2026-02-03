use std::sync::Arc;
use logger::error;
use uuid::Uuid;

use crate::{configuration::Configuration, services::{DocumentsService, SSEService}};

pub struct Services
{
    pub sse_service: SSEService,
    pub documents_service: Arc<DocumentsService>,
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
        let configuration = Arc::new(Configuration::load());
        let sse_service = SSEService::new();
        let documents_service = Arc::new(DocumentsService::default());
        let services = Services
        {
            documents_service,
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
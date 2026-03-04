use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::services::{JWTService, UsersRepository, new_connection};

pub struct AppState
{
    pub jwt_service: JWTService,
    pub users_repository: UsersRepository,
    pub configuration: Arc<crate::configuration::Configuration>,
    pub http_client: reqwest::Client,
    /// Счётчики round-robin для каждого сервиса (ключ — prefix)
    pub counters: HashMap<String, Arc<AtomicUsize>>,
}

impl AppState
{
    pub async fn new() -> Result<Self, crate::error::Error>
    {
        let configuration = Arc::new(crate::configuration::Configuration::new()?);

        let counters = configuration.services.iter()
            .map(|s| (s.prefix.clone(), Arc::new(AtomicUsize::new(0))))
            .collect();

        let connection = Arc::new(new_connection("users.db").await?);
        Ok(Self
        {
            jwt_service: JWTService::new()?,
            users_repository: UsersRepository::new(connection).await?,
            configuration,
            http_client: reqwest::Client::new(),
            counters,
        })
    }
}

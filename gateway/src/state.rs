use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crate::services::{JWTService, ServicesRepository, UsersRepository, new_connection};

pub struct AppState
{
    pub jwt_service: JWTService,
    pub users_repository: UsersRepository,
    pub services_repository: ServicesRepository,
    pub configuration: Arc<crate::configuration::Configuration>,
    pub http_client: reqwest::Client,
    /// Счётчики round-robin (ключ — prefix сервиса), заполняются лениво
    pub counters: Mutex<HashMap<String, Arc<AtomicUsize>>>,
    /// Активные PoW-challenge (ключ — hex-строка, значение — момент создания)
    pub challenges: Mutex<HashMap<String, Instant>>,
}

impl AppState
{
    pub async fn new() -> Result<Self, crate::error::Error>
    {
        let configuration = Arc::new(crate::configuration::Configuration::new()?);
        let connection = Arc::new(new_connection("gateway.db").await?);
        Ok(Self
        {
            jwt_service: JWTService::new()?,
            users_repository: UsersRepository::new(Arc::clone(&connection)).await?,
            services_repository: ServicesRepository::new(Arc::clone(&connection)).await?,
            configuration,
            http_client: reqwest::Client::new(),
            counters: Mutex::new(HashMap::new()),
            challenges: Mutex::new(HashMap::new()),
        })
    }
}

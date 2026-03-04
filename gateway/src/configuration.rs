use anyhow::Context;
use figment::{Figment, providers::{Format, Toml}};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Конфигурация шлюза.
/// Шлюз слушает на `port`, принимает запросы вида `/{prefix}/{path}`
/// и перенаправляет их на один из `targets` нужного сервиса,
/// предварительно проверив права пользователя по JWT.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration
{
    #[serde(default = "port")]
    pub port: u16,
    pub services: Vec<ServiceConfiguration>,
}
impl Configuration
{
    /// Поиск сервиса по URL-префиксу (первый сегмент пути запроса)
    pub fn get_service_by_prefix(&self, prefix: &str) -> Option<&ServiceConfiguration>
    {
        self.services.iter().find(|s| s.prefix == prefix)
    }
}
fn port() -> u16
{
    8080
}

/// Конфигурация одного проксируемого сервиса.
///
/// Пример запроса: `192.168.0.1:8080/info/search`
/// - `prefix`  = `"info"`  — сегмент URL, по которому шлюз определяет сервис
/// - `targets` = `["http://192.168.0.2:8082"]` — адреса бэкенда (при нескольких — round-robin)
/// - `permissions` — список прав, хотя бы одно из которых должно быть у пользователя
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfiguration
{
    /// URL-сегмент, идентифицирующий сервис (без слешей, например `"info"`)
    pub prefix: String,
    /// Имя сервиса — для сопоставления с правами из JWT
    pub service_name: String,
    /// Адреса бэкенд-инстансов сервиса (если несколько)
    pub targets: Vec<String>,
    /// Права, необходимые для доступа к сервису
    pub permissions: Vec<String>,
}

impl Configuration
{
    pub fn new() -> Result<Self, Error>
    {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Toml::file("/config/config.toml"))
            .extract::<Self>()
            .map_err(|e| Error::ConfigurationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests
{
    use crate::logger;

    #[test]
    fn load_cfg()
    {
        logger::init();
        let cfg = super::Configuration::new().expect("No cfg!");
    }

    #[test]
    fn save_cfg()
    {
        logger::init();
        let cfg = super::Configuration::new().expect("No cfg!");
        utilites::serialize(cfg, "configuration.toml", false, utilites::Serializer::Toml);
    }
}

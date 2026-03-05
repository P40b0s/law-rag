use anyhow::Context;
use figment::{Figment, providers::{Format, Toml}};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Конфигурация шлюза.
/// Шлюз слушает на `port`, принимает запросы вида `/{prefix}/{path}`
/// и перенаправляет их на один из `targets` нужного сервиса,
/// предварительно проверив права пользователя по JWT.
/// Конфигурация шлюза.
/// Сервисы хранятся в БД и редактируются через API `/gateway-admin/services`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration
{
    #[serde(default = "port")]
    pub port: u16,
}

fn port() -> u16
{
    8090
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

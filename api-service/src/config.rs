use crate::metrics::MetricsConfig;
use crate::server::ServerConfig;
use cache_service::FileCacheConfig;
use channel_cache::ChannelCacheConfig;
use client_backend::BackendClientConfig;
use client_tg::TgConfig;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub metrics: Option<MetricsConfig>,
    // pub file_cache: FileCacheConfig,
    #[serde(default)]
    pub server: ServerConfig,
    pub tg: TgConfig,
    pub backend: BackendClientConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub file_cache: FileCacheConfig,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct CacheConfig {
    #[serde(default)]
    pub channel: ChannelCacheConfig,
}

impl Configuration {
    pub fn new() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Toml::file("/config/config.toml"))
            .merge(Env::prefixed("APP_"))
            .extract()
    }
}

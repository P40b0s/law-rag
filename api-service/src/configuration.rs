use std::{path::{Path, PathBuf}, sync::Arc};

use anyhow::Context;
use figment::{Figment, providers::{Env, Format, Toml}};
use rag_core::{EmbeddingConfiguration, QdrantConfiguration};
use tracing::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration
{
    #[serde(default = "origins")]
    pub origins: Vec<String>,
    #[serde(default = "port")]
    pub server_port: u16,
    #[serde(default)]
    pub embedding_configuration: Arc<EmbeddingConfiguration>,
    #[serde(default)]
    pub qdrant_configuration: Arc<QdrantConfiguration>
}


fn origins() -> Vec<String>
{
    vec![
                "http://localhost:8080".to_owned(),
                "http://localhost:8081".to_owned(),
                "http://localhost:80".to_owned(),
            ]
}
fn port() -> u16
{
    8081
}


impl Configuration
{
    pub fn new() -> Result<Self, anyhow::Error> 
    {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Toml::file("/config/config.toml"))
            .merge(Env::prefixed("RAG_"))
            .extract::<Self>()
            .context("Ошибка инициализации настроек программы")
    }

    // pub fn add_origin(mut self) -> Self
    // {
    //     if let Some(addr) = get_local_ip()
    //     {
    //         let addr = format!("http://{}:{}", addr.to_string(), self.server_port);
    //         info!("frontend started on addresse: {}", &addr);
    //         self.origins.push(addr);
    //     }
    //     self
    // }
}

#[cfg(test)]
mod tests
{
    use tracing::info;

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
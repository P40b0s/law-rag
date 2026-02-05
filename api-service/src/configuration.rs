use std::{path::{Path, PathBuf}, sync::Arc};

use anyhow::Context;
use embedding::EmbeddingConfiguration;
use figment::{Figment, providers::{Env, Format, Toml}};
use tracing::info;
use serde::{Deserialize, Serialize};
use crate::services::get_local_ip;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration
{
    #[serde(default = "origins")]
    pub origins: Vec<String>,
    #[serde(default = "port")]
    pub server_port: u16,
    #[serde(default)]
    pub embedding_configuration: Arc<EmbeddingConfiguration>,
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
            .and_then(|c| Ok(c.add_origin()))
            .context("Ошибка инициализации настроек программы")
    }

    pub fn add_origin(mut self) -> Self
    {
        if let Some(addr) = get_local_ip()
        {
            let addr = format!("http://{}:{}", addr.to_string(), self.server_port);
            info!("frontend started on addresse: {}", &addr);
            self.origins.push(addr);
        }
        self
    }
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
       info!("{}", cfg.embedding_configuration.generator_config_path.display());

    }
}
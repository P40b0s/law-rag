use std::path::{Path, PathBuf};

use logger::info;
use serde::{Deserialize, Serialize};

use crate::services::get_local_ip;

const FILENAME: &str = "configuration.toml";


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilesConfiguration
{
    pub files_directory: PathBuf,
    pub max_filesize_mb: usize,
    pub tasks_directory: PathBuf,
    pub root_directory: PathBuf
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdminAccessConfiguration
{
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration
{
    ///access key lifetime un minutes
    pub access_key_lifetime: u16,
    pub origins: Vec<String>,
    pub server_port: u16,
    pub files_configuration: FilesConfiguration,
    pub admin_access_configuration: AdminAccessConfiguration

}
impl Default for FilesConfiguration
{
    fn default() -> Self 
    {
        let current_dir = std::env::current_dir().unwrap_or( Path::new("").to_path_buf());
        let files_directory = Path::new("files").to_path_buf();
        let tasks_directory = 
        {
            let mut cloned_files_directory = files_directory.clone();
            cloned_files_directory.push("tasks");
            cloned_files_directory
        };
        Self 
        { 
            files_directory,
            max_filesize_mb: 200,
            tasks_directory,
            root_directory: current_dir
        }
    }
}
impl Default for Configuration
{
    fn default() -> Self 
    {
        let server_port = 8081;
        Self
        {
            //30 дней
            access_key_lifetime: 43200,
            origins: vec![
                "http://localhost:8080".to_owned(),
                "http://localhost:8081".to_owned(),
                "http://localhost:80".to_owned(),
            ],
            server_port,
            files_configuration: FilesConfiguration::default(),
            admin_access_configuration: AdminAccessConfiguration 
            {
                username: "admin".to_owned(),
                password: "admin".to_owned(),
            }
        }
    }
}
impl Configuration
{
    pub fn load() -> Self
    {
        let cfg = utilites::deserialize(FILENAME, false, utilites::Serializer::Toml);
        let mut cfg = if cfg.is_err()
        {
            logger::error!("Ошибка десериализации настроек, {}, будут установлены настройки по умолчанию", cfg.err().unwrap());
            Self::default()
        }
        else 
        {
            cfg.unwrap()    
        };
        //add current host to origins
        if let Some(addr) = get_local_ip()
        {
            let addr = format!("http://{}:{}", addr.to_string(), cfg.server_port);
            info!("frontend started on addresse: {}", &addr);
            cfg.origins.push(addr);
        }
        cfg
    }
    pub fn save(&self)
    {
        let _ = utilites::serialize(&self, FILENAME, false, utilites::Serializer::Toml);
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn save_cfg()
    {
        let cfg = super::Configuration::default();
        cfg.save();
    }
}
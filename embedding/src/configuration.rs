use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct EmbeddingConfiguration 
{
    #[serde(default = "reranker_model_path")]
    pub reranker_model_path: PathBuf,
    #[serde(default = "reranker_config_path")]
    pub reranker_config_path: PathBuf,
    #[serde(default = "reranker_tokenizer_path")]
    pub reranker_tokenizer_path: PathBuf,
    
    #[serde(default = "retriver_model_path")]
    pub retriver_model_path: PathBuf,
    #[serde(default = "retriver_config_path")]
    pub retriver_config_path: PathBuf,
    #[serde(default = "retriver_tokenizer_path")]
    pub retriver_tokenizer_path: PathBuf,

    #[serde(default = "generator_model_path")]
    pub generator_model_path: PathBuf,
    #[serde(default = "generator_config_path")]
    pub generator_config_path: PathBuf,
    #[serde(default = "generator_tokenizer_path")]
    pub generator_tokenizer_path: PathBuf,
}

fn reranker_model_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/reranker/model.safetensors").to_path_buf()
}
fn reranker_config_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/reranker/config.json").to_path_buf()
}
fn reranker_tokenizer_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/reranker/tokenizer.json").to_path_buf()
}

fn retriver_model_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/pytorch_model.bin").to_path_buf()
}
fn retriver_config_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/config.json").to_path_buf()
}
fn retriver_tokenizer_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/tokenizer.json").to_path_buf()
}

fn generator_model_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/generator/llama3/model-q4_K.gguf").to_path_buf()
}
fn generator_config_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/generator/llama3/config.json").to_path_buf()
}
fn generator_tokenizer_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/generator/llama3/tokenizer.json").to_path_buf()
}


impl Default for EmbeddingConfiguration
{
    fn default() -> Self 
    {
        Self 
        { 
            reranker_model_path: reranker_model_path(),
            reranker_config_path: reranker_config_path(),
            reranker_tokenizer_path: reranker_tokenizer_path(),
            retriver_model_path: retriver_model_path(),
            retriver_config_path: retriver_config_path(),
            retriver_tokenizer_path: retriver_tokenizer_path(),
            generator_model_path: generator_model_path(),
            generator_config_path: generator_config_path(),
            generator_tokenizer_path: generator_tokenizer_path()
        }
    }
}

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
    #[serde(default = "system_prompt")]
    pub system_prompt: String,
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
    Path::new("/home/phobos/projects/rust/law-rag/model/generator/llama3/1b/model.safetensors").to_path_buf()
}
fn generator_config_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/generator/llama3//1b/config.json").to_path_buf()
}
fn generator_tokenizer_path() -> PathBuf 
{
    Path::new("/home/phobos/projects/rust/law-rag/model/generator/llama3/1b/tokenizer.json").to_path_buf()
}
fn system_prompt() -> String
{
    "Ты - высококвалифицированный юрист, который специализируется на российском законодательстве. Ты помогаешь людям находить ответы на их вопросы, используя свои обширные знания законов и нормативных актов. Ты всегда предоставляешь точные и подробные ответы, ссылаясь на конкретные статьи и пункты законодательства. Ты также можешь объяснять сложные юридические концепции простым языком, чтобы помочь людям лучше понять их права и обязанности. Твоя цель - помочь людям разобраться в их юридических вопросах и предоставить им полезную информацию. В конце добавь ссылки на документы откуда ты взял информацию, а так же полный путь откуда взята информация (пнкт подпункт статья итд.)  Ответ выдавай в mardown формате".to_owned()
}
//"Ты - ассистент RAG системы, ты должен отвечать на вопросы пользователей используя ТОЛЬКО предоставленный контекст для формирования ответа. начало ответа должно звучать так: `На основе имеющейся у меня информации: {далее идет твой ответ}`.  Если в контексте нет информации, скажи: \"Не могу ответить на основе имеющейся информации\"".to_owned(),


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
            generator_tokenizer_path: generator_tokenizer_path(),
            system_prompt: system_prompt(),
        }
    }
}

use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr};
use anyhow::anyhow;
/// Имя модели с типом
#[derive(Deserialize, Clone, Debug, Serialize, PartialEq)]
pub enum ModelName
{
    BgeM3,
    BgeRerankerV2M3,
    Llama1b,
    Llama8b,
    Qwen32b,
    Qwen3_9b
}

impl AsRef<str> for ModelName
{
    fn as_ref(&self) -> &str 
    {
        match self
        {
            ModelName::BgeM3 => "BAAI/bge-m3",
            ModelName::BgeRerankerV2M3 => "BAAI/bge-reranker-v2-m3",
            ModelName::Llama1b => "meta-llama/Llama-3.2-1B-Instruct" ,
            ModelName::Llama8b => "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF",
            ModelName::Qwen32b =>  "Qwen/Qwen2.5-32B-Instruct",
            ModelName::Qwen3_9b => "Qwen/Qwen3.5-9B"
        }
    }
}
impl Display for ModelName
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        f.write_str(self.as_ref())
    }
}

impl FromStr for ModelName 
{
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match s 
        {
            "meta-llama/Llama-3.2-1B-Instruct" => Ok(ModelName::Llama1b),
            "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF" => Ok(ModelName::Llama8b),
            "Qwen/Qwen2.5-32B-Instruct" => Ok(ModelName::Qwen32b),
            "BAAI/bge-reranker-v2-m3" => Ok(ModelName::BgeRerankerV2M3),
            "BAAI/bge-m3" => Ok(ModelName::BgeM3),
            "Qwen/Qwen3.5-9B" => Ok(ModelName::Qwen3_9b),
            _ => Err(anyhow!("Модель {} не определена в конфигурационном файле", s))
        }
    }    
}
impl ModelName
{
    pub fn retriver_bgem3() -> Self
    {
        Self::BgeM3
    }
    pub fn reranker_bge_v2_m3() -> Self
    {
        Self::BgeRerankerV2M3
    }
    pub fn generator_llama_1b() -> Self
    {
        Self::Llama1b
    }
    pub fn generator_llama_8b() -> Self
    {
        Self::Llama8b
    }
    pub fn generator_external_qwen32b() -> Self
    {
        Self::Qwen32b
    }
    pub fn generator_external_qwen3_9b() -> Self
    {
        Self::Qwen3_9b
    }
}

/// Пути к файлам локальной модели (weights, config, tokenizer) + идентификатор.
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct LocalModelConfig
{
    #[serde(deserialize_with="model_name_deserializer")]
    pub name:           ModelName,
    pub model_file:     String,
    pub model_path:     PathBuf,
    pub config_path:    PathBuf,
    pub tokenizer_path: PathBuf,
    //на случай если известно и нужно
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<usize>,
}

fn model_name_deserializer<'de, D>(deserializer: D) -> Result<ModelName, D::Error>
where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    s.parse::<ModelName>().map_err(serde::de::Error::custom)
}

/// Конфигурация конкретного генератора — локальная модель или внешний сервер.
#[derive(Deserialize, Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeneratorConfig
{
    /// Llama 3.2 1B (локальный, safetensors)
    Llama1b(LocalModelConfig),
    /// Llama 3.1 8B (локальный, gguf)
    Llama8b(LocalModelConfig),
    /// Внешний OpenAI-совместимый сервер (llama.cpp, Ollama, vLLM)
    External
    {
        name:     ModelName,
        base_url: String,
    },
}

impl GeneratorConfig
{
    pub fn name(&self) -> &ModelName
    {
        match self
        {
            Self::Llama1b(cfg) 
            | Self::Llama8b(cfg) => &cfg.name,
            Self::External { name, .. }             => name,
        }
    }

    pub fn model_path(&self) -> Option<&PathBuf>
    {
        match self
        {
            Self::Llama1b(cfg) 
            | Self::Llama8b(cfg) => Some(&cfg.model_path),
            Self::External { .. }                   => None,
        }
    }

    pub fn config_path(&self) -> Option<&PathBuf>
    {
        match self
        {
            Self::Llama1b(cfg) 
            | Self::Llama8b(cfg) => Some(&cfg.config_path),
            Self::External { .. }                   => None,
        }
    }

    pub fn tokenizer_path(&self) -> Option<&PathBuf>
    {
        match self
        {
            Self::Llama1b(cfg) 
            | Self::Llama8b(cfg) => Some(&cfg.tokenizer_path),
            Self::External { .. }                   => None,
        }
    }
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct EmbeddingConfiguration
{
    /// Доступные reranker-модели
    #[serde(default = "default_rerankers")]
    pub rerankers: Vec<LocalModelConfig>,

    /// Доступные retriever-модели
    #[serde(default = "default_retrievers")]
    pub retrievers: Vec<LocalModelConfig>,

    /// Доступные генераторы
    #[serde(default = "default_generators")]
    pub generators: Vec<GeneratorConfig>,

    /// Размер мини-батча для эмбеддингов
    #[serde(default = "default_embedding_batch_size")]
    pub embedding_batch_size: usize,

    /// Минимальный score реранкера для выдачи результата
    #[serde(default = "default_min_reranking_score")]
    pub min_reranking_score: f32,

    /// Системный промпт для генератора
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
    /// Температура генератора
    #[serde(default = "default_generator_temperature")]
    pub generator_temperature: f64,
}

impl EmbeddingConfiguration
{
    pub fn find_retriever(&self, name: &ModelName) -> anyhow::Result<&LocalModelConfig>
    {
        self.retrievers.iter().find(|r| &r.name == name)
            .ok_or(anyhow!("Модель {} отсуствует в текущей конфигурации", name))
    }

    pub fn find_reranker(&self, name: &ModelName) -> anyhow::Result<&LocalModelConfig>
    {
        self.rerankers.iter().find(|r| &r.name == name)
            .ok_or(anyhow!("Модель {} отсуствует в текущей конфигурации", name))
    }

    pub fn find_generator(&self, name: &ModelName) -> anyhow::Result<&GeneratorConfig>
    {
        self.generators.iter().find(|r| r.name() == name)
            .ok_or(anyhow!("Модель {} отсуствует в текущей конфигурации", name))
    }
}

// ── defaults ─────────────────────────────────────────────────────────────────

fn default_rerankers() -> Vec<LocalModelConfig>
{
    vec![LocalModelConfig
    {
        name:           ModelName::BgeRerankerV2M3,
        model_file:     "model.safetensors".to_owned(),
        model_path:     PathBuf::from("/home/phobos/projects/rust/law-rag/model/reranker/model.safetensors"),
        config_path:    PathBuf::from("/home/phobos/projects/rust/law-rag/model/reranker/config.json"),
        tokenizer_path: PathBuf::from("/home/phobos/projects/rust/law-rag/model/reranker/tokenizer.json"),
        dimension:      Some(1), // Реранкеры обычно возвращают один скор релевантности на пару (query, document)
    }]
}

fn default_retrievers() -> Vec<LocalModelConfig>
{
    vec![LocalModelConfig
    {
        name:           ModelName::BgeM3,
        model_file:     "pytorch_model.bin".to_owned(),
        model_path:     PathBuf::from("/home/phobos/projects/rust/law-rag/model/pytorch_model.bin"),
        config_path:    PathBuf::from("/home/phobos/projects/rust/law-rag/model/config.json"),
        tokenizer_path: PathBuf::from("/home/phobos/projects/rust/law-rag/model/tokenizer.json"),
        dimension:      None, //берется из конфигурации модели из поля hidden_size
    }]
}

fn default_generators() -> Vec<GeneratorConfig>
{
    vec![
        GeneratorConfig::Llama1b(LocalModelConfig
        {
            name:           ModelName::Llama1b,
            model_file:     "model.safetensors".to_owned(),
            model_path:     PathBuf::from("/home/phobos/projects/rust/law-rag/model/generator/llama3/1b/model.safetensors"),
            config_path:    PathBuf::from("/home/phobos/projects/rust/law-rag/model/generator/llama3/1b/config.json"),
            tokenizer_path: PathBuf::from("/home/phobos/projects/rust/law-rag/model/generator/llama3/1b/tokenizer.json"),
            dimension:      None,
        }),
        GeneratorConfig::Llama8b(LocalModelConfig
        {
            name:           ModelName::Llama8b,
            model_file:     "model-q4_K.gguf".to_owned(),
            model_path:     PathBuf::from("/home/phobos/projects/rust/law-rag/model/generator/llama3/model-q4_K.gguf"),
            config_path:    PathBuf::from("/home/phobos/projects/rust/law-rag/model/generator/llama3/config.json"),
            tokenizer_path: PathBuf::from("/home/phobos/projects/rust/law-rag/model/generator/llama3/tokenizer.json"),
            dimension:      None,
        }),
        GeneratorConfig::External 
        { 
            name: ModelName::Qwen3_9b, 
            base_url: "http://10.0.0.2:8000".to_owned()
        }
    ]
}

fn default_generator_temperature() -> f64 { 0.1 }
fn default_embedding_batch_size() -> usize { 8 }
fn default_min_reranking_score()  -> f32   { 0.7 }

fn default_system_prompt() -> String
{
    "Ты - высококвалифицированный юрист, который специализируется на российском законодательстве. 
     Ты помогаешь людям находить ответы на их вопросы, используя свои обширные знания законов и 
     нормативных актов. Ты всегда предоставляешь точные и подробные ответы, ссылаясь на конкретные 
     статьи и пункты законодательства. Ты также можешь объяснять сложные юридические концепции 
     простым языком, чтобы помочь людям лучше понять их права и обязанности. Твоя цель - помочь 
     людям разобраться в их юридических вопросах и предоставить им полезную информацию. В конце 
     добавь ссылки на документы откуда ты взял информацию, а так же полный путь откуда взята 
     информация (пнкт подпункт статья итд.) Ответ выдавай в markdown формате".to_owned()
}

impl Default for EmbeddingConfiguration
{
    fn default() -> Self
    {
        Self
        {
            rerankers:            default_rerankers(),
            retrievers:           default_retrievers(),
            generators:           default_generators(),
            embedding_batch_size: default_embedding_batch_size(),
            min_reranking_score:  default_min_reranking_score(),
            system_prompt:        default_system_prompt(),
            generator_temperature:default_generator_temperature(),
        }
    }
}

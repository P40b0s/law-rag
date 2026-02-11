use std::fmt::Display;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ServiceStatus
{
    pub hash: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub current_chunk: Option<usize>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub overall_chunks: Option<usize>,
    pub status: ProcessStatus,
    pub message: String,
}
#[derive(Debug, Serialize, Clone, Copy)]
pub enum ProcessStatus
{
    Embedding,
    Reranking,
    Generation,
    Chunking,
    Message,
    Complete,
    Error
}


impl ServiceStatus
{
    ///текущий процесс добавления чанков в бд qdrant
    /// ембеддинг и добавление
    pub fn process_qdrant(hash: &str, current: usize, count: usize) -> Self
    {
        Self 
        { 
            hash: hash.to_owned(),
            current_chunk: Some(current),
            overall_chunks: Some(count),
            message: format!("Обработка фрагмента {}/{}", current, count),
            status: ProcessStatus::Embedding,
        }
    }
    ///процесс формирования чанков из целевого объекта (пока html)
    pub fn process_chunk(hash: &str, current: usize, count: usize) -> Self
    {
        Self 
        { 
            hash: hash.to_owned(),
            current_chunk: Some(current),
            overall_chunks: Some(count),
            message: format!("Обработка параграфа {}/{}", current, count),
            status: ProcessStatus::Chunking,
        }
    }
    pub fn error<E: Display>(doc_uri: String, error: E) -> Self
    {
        let error = format!("Произошла ошибка при обработке документа {} -> {}", doc_uri, error);
        Self 
        { 
            hash: String::new(),
            current_chunk: None,
            overall_chunks: None,
            message: error,
            status: ProcessStatus::Error
        }
    }
    pub fn mesage(msg: String) -> Self
    {
        Self 
        { 
            hash: String::new(),
            current_chunk: None,
            overall_chunks: None,
            message: msg,
            status: ProcessStatus::Message
        }
    }
    pub fn process_generation(msg: String) -> Self
    {
        Self 
        { 
            hash: String::new(),
            current_chunk: None,
            overall_chunks: None,
            message: msg,
            status: ProcessStatus::Generation
        }
    }
    pub fn complete(hash: &str) -> Self
    {
        Self 
        { 
            hash: hash.to_owned(),
            current_chunk: None,
            overall_chunks: None,
            message: format!("Процесс для документа {} завершен", hash),
            status: ProcessStatus::Complete
        }
    }
    pub fn qdrant_error<E: Display>(error: E) -> Self
    {
        let error = format!("Произошла ошибка при попытке использования базы данных qdrant: {}", error);
        Self 
        { 
            hash: String::new(),
            current_chunk: None,
            overall_chunks: None,
            message: error,
            status: ProcessStatus::Error
        }
    }
}
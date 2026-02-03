use embedding::Chunk;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ChunkProcess
{
    pub hash: String,
    pub current_chunk: usize,
    pub overall_chunks: usize,
    pub status: String,
    pub is_error: bool
}
impl ChunkProcess
{
    ///текущий процесс добавления чанков в бд qdrant
    pub fn process_qdrant(hash: &str, current: usize, count: usize) -> Self
    {
        Self 
        { 
            hash: hash.to_owned(),
            current_chunk: current,
            overall_chunks: count,
            status: format!("Обработка фрагмента {}/{}", current, count),
            is_error: false
        }
    }
    ///текущий процесс обработки чанков
    pub fn process_chunk(hash: &str, current: usize, count: usize) -> Self
    {
        Self 
        { 
            hash: hash.to_owned(),
            current_chunk: current,
            overall_chunks: count,
            status: format!("Обработка параграфа {}/{}", current, count),
            is_error: false
        }
    }
    pub fn error(doc_uri: String, error: crate::Error) -> Self
    {
        let error = format!("Произошла ошибка при обработке документа {} -> {}", doc_uri, error);
        Self 
        { 
            hash: String::new(),
            current_chunk: 0,
            overall_chunks: 0,
            status: error,
            is_error: true
        }
    }
    pub fn qdrant_error(error: crate::Error) -> Self
    {
        let error = format!("Произошла ошибка при попытке использования базы данных qdrant: {}", error);
        Self 
        { 
            hash: String::new(),
            current_chunk: 0,
            overall_chunks: 0,
            status: error,
            is_error: true
        }
    }
}
use database::SearchResult;
use rag_core::{Chunk, RerankResult};
use serde::Serialize;
use utilites::Date;

#[derive(Debug, Serialize)]
pub struct QdrantContext
{
    pub original_score: f32,              // Оригинальный скор от Qdrant
    pub reranker_score: f32,       // Скор после переоценки
    pub id: String,
    pub text: String,                // Оригинальный текст чанка
    pub document_uri: String,        // URI исходного документа
    pub document_hash: String,        // URI исходного документа
    pub document_title: String, // Заголовки документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: String, // Дата подписания документа
    pub path: String, // Полный путь
}

impl From<RerankResult<SearchResult>> for QdrantContext
{
    fn from(result: RerankResult<SearchResult>) -> Self 
    {
        let db_object = result.db_object;
        Self
        {
            id: db_object.id.clone(),
            original_score: db_object.score,
            reranker_score: result.score,
            text: db_object.payload.text.clone(),
            path: db_object.payload.path.clone(),
            document_uri: db_object.payload.document_uri.clone(),
            document_hash: db_object.payload.document_hash.clone(),
            document_title: db_object.payload.document_title.clone(),
            document_number: db_object.payload.document_number.clone(),
            document_sign_date: db_object.payload.sign_date_as_str(),
        }
    }
}

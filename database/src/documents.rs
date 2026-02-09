use std::{pin::Pin, sync::Arc};

use embedding::Chunk;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool, prelude::FromRow, sqlite::SqliteRow};
use tracing::{error, warn};
use utilites::Date;
use anyhow::{Context, Result};
use crate::{Error, connection};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentDbo
{
    pub document_uri: String,        // URI исходного документа
    pub document_hash: String,
    pub document_title: String, // Заголовок документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: Date, // Дата подписания документа
    pub has_embeddings: bool,
    pub chunks_count: u32,
    pub chunks: Vec<Chunk>,
    pub status: DocumentStatus,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentCardDbo
{
    pub document_uri: String,
    pub document_hash: String,
    pub document_title: String,
    pub document_number: String, 
    pub document_sign_date: Date,
    pub has_embeddings: bool,
    pub chunks_count: u32,
    pub status: DocumentStatus,
}

impl FromRow<'_, SqliteRow> for DocumentDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let document_uri: String =  row.try_get("document_uri")?;
        let document_title: String =  row.try_get("document_title")?;
        let document_number: String =  row.try_get("document_number")?;
        let document_hash: String =  row.try_get("document_hash")?;
        let document_sign_date: &str =  row.try_get("document_sign_date")?;
        let document_sign_date = document_sign_date.parse().unwrap();
        let has_embeddings: bool =  row.try_get("has_embeddings")?;
        let chunks_count: u32 = row.try_get("chunks_count")?;
        let chunks: &str = row.try_get("chunks")?;
        let chunks = serde_json::from_str(chunks).unwrap_or_default();
        let status: u8 =  row.try_get("status")?;
        let obj = DocumentDbo
        {
            document_uri,
            document_title,
            document_number,
            document_hash,
            document_sign_date,
            has_embeddings,
            chunks_count,
            status: status.into(),
            chunks
        };
        Ok(obj)
    }
}

impl FromRow<'_, SqliteRow> for DocumentCardDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let document_uri: String =  row.try_get("document_uri")?;
        let document_title: String =  row.try_get("document_title")?;
        let document_number: String =  row.try_get("document_number")?;
        let document_hash: String =  row.try_get("document_hash")?;
        let document_sign_date: &str =  row.try_get("document_sign_date")?;
        let document_sign_date = document_sign_date.parse().unwrap();
        let has_embeddings: bool =  row.try_get("has_embeddings")?;
        let chunks_count: u32 = row.try_get("chunks_count")?;
        let status: u8 =  row.try_get("status")?;
        let obj = DocumentCardDbo
        {
            document_uri,
            document_title,
            document_number,
            document_hash,
            document_sign_date,
            has_embeddings,
            chunks_count,
            status: status.into(),
        };
        Ok(obj)
    }
}

pub struct DocumentsTable
{
    connection: Arc<SqlitePool>,
}

impl DocumentsTable
{
     fn create_code() -> &'static str 
    {
        "BEGIN;
        CREATE TABLE IF NOT EXISTS documents (
        document_uri TEXT NOT NULL,
        document_title TEXT NOT NULL,
        document_hash TEXT NOT NULL,
        document_number TEXT NOT NULL,
        document_sign_date TEXT NOT NULL,
        status INTEGER NOT NULL,
        has_embeddings BOOLEAN NOT NULL DEFAULT (0),
        chunks_count INTEGER NOT NULL DEFAULT (0),
        chunks TEXT NOT NULL DEFAULT('[]'),
        PRIMARY KEY(document_hash)
        );
        CREATE INDEX IF NOT EXISTS 'document_uri_idx' ON documents (document_uri);
        CREATE INDEX IF NOT EXISTS 'document_number_idx' ON documents (document_number);
        CREATE INDEX IF NOT EXISTS 'document_sign_date_idx' ON documents (document_sign_date);
        COMMIT;"
    }
    fn name() -> &'static str 
    {
        "documents"
    }

    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self>
    {
        let r1 = sqlx::query(DocumentsTable::create_code()).execute(&*pool).await;
        if r1.is_err()
        {
            error!("{}", r1.as_ref().err().unwrap());
            let _ = r1?;
        };
        Ok(Self
        {
            connection: pool,
        })
    }
    pub async fn new_default() -> Result<Self>
    {
        let pool = connection::new_connection("documents").await?;
        let table = super::DocumentsTable::new(Arc::new(pool)).await?;
        Ok(table)
    }

    pub fn get_document_by_hash<'a>(&'a self, doc_hash: &'a str) -> Pin<Box<dyn Future<Output = Result<DocumentDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,DocumentsTable::name()
                ," WHERE "
                ,"document_hash = $1"
            ].concat();
            let doc = sqlx::query_as::<_, DocumentDbo>(&sql)
            .bind(doc_hash)
            .fetch_one(&*connection).await;
            let doc = doc.context(format!("Document {} not found in database", &doc_hash))?;
            Ok(doc)
        })
    }
    pub fn get_documents<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Vec<DocumentDbo>>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,DocumentsTable::name()
            ].concat();
            let docs: Vec<DocumentDbo> = sqlx::query_as::<_, DocumentDbo>(&sql)
            .fetch_all(&*connection).await?;
            Ok(docs)
        })
    }
    pub fn get_documents_without_chunks<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Vec<DocumentCardDbo>>> + Send + 'a>>
    {
        Box::pin(async move
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT  document_uri, document_title, document_hash, document_number, document_sign_date, status, has_embeddings, chunks_count"
                ," FROM "
                ,DocumentsTable::name()
            ].concat();
            let docs: Vec<DocumentCardDbo> = sqlx::query_as::<_, DocumentCardDbo>(&sql)
            .fetch_all(&*connection).await?;
            Ok(docs)
        })
    }

    pub fn get_documents_paginated<'a>(&'a self, limit: u32, offset: u32) -> Pin<Box<dyn Future<Output = Result<Vec<DocumentCardDbo>>> + Send + 'a>>
    {
        Box::pin(async move
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT  document_uri, document_title, document_hash, document_number, document_sign_date, status, has_embeddings, chunks_count"
                ," FROM "
                ,DocumentsTable::name()
                ," ORDER BY document_sign_date DESC"
                ," LIMIT ? OFFSET ?"
            ].concat();
            let docs: Vec<DocumentCardDbo> = sqlx::query_as::<_, DocumentCardDbo>(&sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*connection).await?;
            Ok(docs)
        })
    }

    pub fn get_documents_count<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<i64>> + Send + 'a>>
    {
        Box::pin(async move
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT COUNT(*) as count"
                ," FROM "
                ,DocumentsTable::name()
            ].concat();
            let row = sqlx::query(&sql)
            .fetch_one(&*connection).await?;
            let count: i64 = row.try_get("count")?;
            Ok(count)
        })
    }

    pub fn get_document<'a>(&'a self, sign_date: &'a Date, number: &'a str) -> Pin<Box<dyn Future<Output = Result<DocumentDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,DocumentsTable::name()
                ," WHERE "
                ,"document_sign_date = ?"
                ," AND "
                ,"document_number = ?"
            ].concat();
            let doc = sqlx::query_as::<_, DocumentDbo>(&sql)
            .bind(sign_date.to_string())
            .bind(number)
            .fetch_one(&*connection).await
                .context(format!("Document {} {} not found or founded bigger than 1", sign_date.to_string(), number))?;
            Ok(doc)
        })
    }
    pub fn delete_document<'a>(&'a self, doc_hash: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "DELETE FROM "
                ,DocumentsTable::name()
                ," WHERE "
                ,"document_hash = ?"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(doc_hash)
            .execute(&*connection).await
                .context(format!("Error when delete document {}", doc_hash))?;
            Ok(())
        })
    }
    pub fn update_status<'a>(&'a self, status: DocumentStatus, doc_hash: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        let status: u8 = status.into();
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "UPDATE  "
                ,DocumentsTable::name()
                ," SET status = ? "
                ," WHERE "
                ,"document_hash = ?"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(status)
            .bind(doc_hash)
            .execute(&*connection).await
                .context(format!("Error when updating document {}", doc_hash))?;
            Ok(())
        })
    }
    pub fn update<'a>(&'a self, status: DocumentStatus, has_embeddings: bool, chunks_count: u32, doc_hash: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        let status: u8 = status.into();
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "UPDATE  "
                ,DocumentsTable::name()
                ," SET status = ?, has_embeddings = ?, chunks_count = ? "
                ," WHERE "
                ,"document_hash = ?"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(status)
            .bind(has_embeddings)
            .bind(chunks_count)
            .bind(doc_hash)
            .execute(&*connection).await
                .context(format!("Error when updating document {}", doc_hash))?;
            Ok(())
        })
    }

    pub fn create_document<'a>(
        &'a self,
        uri: &'a str,
        hash: &'a str,
        title: &'a str,
        number: &'a str,
        sign_date: &'a Date,
        status: DocumentStatus,
        has_embeddings: bool,
        chunks_count: &'a u32,
        chunks: &[Chunk]) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        let status: u8 = status.into();
        let chunks = serde_json::to_string(chunks).inspect_err(|e| error!("{e}"))
            .unwrap_or(String::from("'[]'"));
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
           
            let sql = [
                "INSERT INTO "
                ,DocumentsTable::name()
                ," ("
                ,"document_uri, document_hash, document_title, document_number, document_sign_date, status, has_embeddings, chunks_count, chunks"
                ,") VALUES (?,?,?,?,?,?,?,?,json(?))"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(uri)
            .bind(hash)
            .bind(title)
            .bind(number)
            .bind(sign_date.to_string())
            .bind(status)
            .bind(has_embeddings)
            .bind(chunks_count)
            .bind(chunks)
            .execute(&*connection).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use tracing::info;
    use utilites::Date;

    use crate::{connection, logger};
    #[tokio::test]
    async fn test_db()
    {
        let pool = connection::new_connection("documents").await.unwrap();
        let table = super::DocumentsTable::new(Arc::new(pool)).await.unwrap();
        let add_doc = table.create_document(
            "www.test.ru",
            "FE7463FFGEGDGE",
            "тестовый документ",
            "bla-123",
            &Date::now(),
            crate::documents::DocumentStatus::Loaded,
            true,
        &123,
        &vec![]).await.unwrap();
        let get_doc = table.get_document_by_hash("FE7463FFGEGDGE").await.unwrap();
        let del = table.delete_document("FE7463FFGEGDGE").await.unwrap();
    }

    #[tokio::test]
    async fn test_get_docs()
    {
        logger::init();
        let pool = connection::new_connection("documents").await.unwrap();
        let table = super::DocumentsTable::new(Arc::new(pool)).await.unwrap();
        let docs = table.get_documents_paginated(0, 30).await.unwrap();
        info!("{:?}", docs)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DocumentStatus
{
    NotLoaded,
    Loaded,
    Embedded,
    Unknown
}
impl From<DocumentStatus> for u8
{
    fn from(value: DocumentStatus) -> Self 
    {
        match value
        {
            DocumentStatus::NotLoaded => 0,
            DocumentStatus::Loaded => 1,
            DocumentStatus::Embedded => 2,
            DocumentStatus::Unknown => u8::MAX
        }
    }
}

impl From<u8> for DocumentStatus
{
    fn from(value: u8) -> Self 
    {
        match value
        {
            0 => DocumentStatus::NotLoaded,
            1 => DocumentStatus::Loaded,
            2 => DocumentStatus::Embedded,
            _ => DocumentStatus::Unknown,
        }
    }
}
impl AsRef<str> for DocumentStatus
{
    fn as_ref(&self) -> &str 
    {
        match self
        {
            DocumentStatus::Loaded => "Loaded",
            DocumentStatus::Embedded => "Embedded",
            DocumentStatus::NotLoaded => "NotLoaded",
            DocumentStatus::Unknown => "Unknown"
        }
    }
}

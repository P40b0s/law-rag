use std::{pin::Pin, sync::Arc};

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool, prelude::FromRow, sqlite::SqliteRow};
use tracing::{error, warn};
use utilites::Date;
use anyhow::{Context, Result};
use crate::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd)]
pub struct DocumentDbo
{
    pub document_uri: String,        // URI исходного документа
    pub document_title: String, // Заголовок документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: Date, // Дата подписания документа
    pub path: String, // Полный путь
}
impl FromRow<'_, SqliteRow> for DocumentDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let document_uri: String =  row.try_get("document_uri")?;
        let document_title: String =  row.try_get("document_title")?;
        let document_number: String =  row.try_get("document_number")?;
        let document_sign_date: &str =  row.try_get("document_sign_date")?;
        let document_sign_date = document_sign_date.parse().unwrap();
        let path: String =  row.try_get("path")?;
        let obj = DocumentDbo   
        {
            document_uri,
            document_title,
            document_number,
            document_sign_date,
            path
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
        document_number TEXT NOT NULL,
        document_sign_date TEXT NOT NULL,
        path TEXT NOT NULL,
        PRIMARY KEY(document_uri)
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

    pub fn get_document_by_uri<'a>(&'a self, doc_uri: &'a str) -> Pin<Box<dyn Future<Output = Result<DocumentDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,DocumentsTable::name()
                ," WHERE "
                ,"document_uri = $1"
            ].concat();
            let doc = sqlx::query_as::<_, DocumentDbo>(&sql)
            .bind(&doc_uri)
            .fetch_one(&*connection).await;
            let doc = doc.context(format!("Document {} not found in database", &doc_uri))?;
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
    pub fn delete_document<'a>(&'a self, doc_uri: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "DELETE FROM "
                ,DocumentsTable::name()
                ," WHERE "
                ,"document_uri = ?"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(doc_uri)
            .execute(&*connection).await
                .context(format!("Error when delete document {}", doc_uri))?;
            Ok(())
        })
    }

    pub fn create_document<'a>(&'a self, uri: &'a str, title: &'a str, number: &'a str, sign_date: &'a Date, path: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "INSERT INTO "
                ,DocumentsTable::name()
                ," ("
                ,"document_uri, document_title, document_number, document_sign_date, path"
                ,") VALUES (?,?,?,?,?)"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(uri)
            .bind(title)
            .bind(number)
            .bind(sign_date.to_string())
            .bind(path)
            .execute(&*connection).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use utilites::Date;

    use crate::connection;
    #[tokio::test]
    async fn test_db()
    {
        let pool = connection::new_connection("documents").await.unwrap();
        let table = super::DocumentsTable::new(Arc::new(pool)).await.unwrap();
        let add_doc = table.create_document("www.test.ru", "тестовый документ", "bla-123", &Date::now(), "a->b->c").await.unwrap();
    }
}
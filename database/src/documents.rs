use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow};
use tracing::error;
use utilites::Date;

use crate::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, FromRow)]
pub struct DocumentDbo
{
    pub document_uri: String,        // URI исходного документа
    pub document_title: String, // Заголовок документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: Date, // Дата подписания документа
    pub path: String, // Полный путь
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

    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self, Error>
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
}
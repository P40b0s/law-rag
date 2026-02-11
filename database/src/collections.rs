use rag_core::{Chunk, DocumentStatus};
use uuid::Uuid;
use std::{pin::Pin, sync::Arc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool, prelude::FromRow, sqlite::SqliteRow};
use tracing::{error, warn};
use utilites::Date;
use anyhow::{Context, Result};
use crate::{Error, connection};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionDbo
{
    pub id: Uuid,
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
}

impl FromRow<'_, SqliteRow> for CollectionDbo 
{
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> 
    {
        let id: String =  row.try_get("id")?;
        let name: String =  row.try_get("name")?;
        let description: String =  row.try_get("description")?;
        let keywords_str: String = row.try_get("keywords")?;
        let keywords = serde_json::from_str(&keywords_str).unwrap_or_default();
        let obj = CollectionDbo
        {
            id: id.parse().unwrap_or_default(),
            name,
            keywords,
            description
        };
        Ok(obj)
    }
}

pub struct CollectionsTable
{
    connection: Arc<SqlitePool>,
}

impl CollectionsTable
{
     fn create_code() -> &'static str 
    {
        "BEGIN;
        CREATE TABLE IF NOT EXISTS collections (
        id TEXT NOT NULL,
        name TEXT NOT NULL UNIQUE,
        description TEXT NOT NULL,
        keywords TEXT NOT NULL,
        PRIMARY KEY(id)
        );
        CREATE INDEX IF NOT EXISTS 'collection_name_idx' ON collections (name);
        COMMIT;"
    }
    fn name() -> &'static str 
    {
        "collections"
    }

    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self>
    {
        let r1 = sqlx::query(Self::create_code()).execute(&*pool).await;
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
    pub async fn new_default(db_name: &str) -> Result<Self>
    {
        let pool = connection::new_connection(db_name).await?;
        let table = Self::new(Arc::new(pool)).await?;
        Ok(table)
    }

    pub fn get_collection_by_name<'a>(&'a self, name: &'a str) -> Pin<Box<dyn Future<Output = Result<CollectionDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,Self::name()
                ," WHERE "
                ,"name = $1"
            ].concat();
            let doc = sqlx::query_as::<_, CollectionDbo>(&sql)
            .bind(name)
            .fetch_one(&*connection).await;
            let doc = doc.context(format!("Collection {} not found in database", &name))?;
            Ok(doc)
        })
    }

    pub fn get_collection_by_id<'a>(&'a self, id: Uuid) -> Pin<Box<dyn Future<Output = Result<CollectionDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,Self::name()
                ," WHERE "
                ,"id = $1"
            ].concat();
            let doc = sqlx::query_as::<_, CollectionDbo>(&sql)
            .bind(id.to_string())
            .fetch_one(&*connection).await;
            let doc = doc.context(format!("Collection {} not found in database", &id))?;
            Ok(doc)
        })
    }

    pub fn get_collections<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<Vec<CollectionDbo>>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,Self::name()
            ].concat();
            let docs: Vec<CollectionDbo> = sqlx::query_as::<_, CollectionDbo>(&sql)
            .fetch_all(&*connection).await?;
            Ok(docs)
        })
    }

    pub fn get_collection<'a>(&'a self, id: Uuid) -> Pin<Box<dyn Future<Output = Result<CollectionDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "SELECT * "
                ," FROM "
                ,Self::name()
                ," WHERE "
                ,"id = ?"
            ].concat();
            let doc = sqlx::query_as::<_, CollectionDbo>(&sql)
            .bind(id.to_string())
            .fetch_one(&*connection).await
                .context(format!("Collection {} not found or founded bigger than 1", id))?;
            Ok(doc)
        })
    }
    pub fn delete_collection<'a>(&'a self, id: Uuid) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "DELETE FROM "
                ,Self::name()
                ," WHERE "
                ,"id = ?"
            ].concat();
            let _ = sqlx::query(&sql)
            .bind(id.to_string())
            .execute(&*connection).await
                .context(format!("Error when delete collection {}", id))?;
            Ok(())
        })
    }
    pub fn update<'a>(&'a self, description: &'a str, keywords: Vec<String>, id: Uuid) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let sql = [
                "UPDATE  "
                ,Self::name()
                ," SET description = ?, keywords = ? "
                ," WHERE "
                ,"id = ?"
            ].concat();
            let keywords_str = serde_json::to_string(&keywords).unwrap_or_default();
            let _ = sqlx::query(&sql)
            .bind(description)
            .bind(keywords_str)
            .bind(id.to_string())
            .execute(&*connection).await
                .context(format!("Error when updating collection {}", id))?;
            Ok(())
        })
    }

    pub fn create_collection<'a>(
        &'a self,
        name: &'a str,
        description: &'a str,
        keywords: Vec<String>,
        ) -> Pin<Box<dyn Future<Output = Result<CollectionDbo>> + Send + 'a>>
    {
        Box::pin(async move 
        {
            let connection = Arc::clone(&self.connection);
            let collection = CollectionDbo
            {
                id: Uuid::now_v7(),
                name: name.to_string(),
                description: description.to_string(),
                keywords
            };
            let sql = [
                "INSERT INTO "
                ,Self::name()
                ," ("
                ,"id, name, description, keywords"
                ,") VALUES (?,?,?,?)"
            ].concat();
            let keywords_str = serde_json::to_string(&collection.keywords).unwrap_or_default();
            let _ = sqlx::query(&sql)
            .bind(collection.id.to_string())
            .bind(name)
            .bind(description)
            .bind(keywords_str)
            .execute(&*connection).await?;
            Ok(collection)
        })
    }
}

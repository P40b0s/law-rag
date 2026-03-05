use std::sync::Arc;
use sqlx::SqlitePool;
use tracing::error;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct ServiceDbo
{
    pub id: i64,
    pub prefix: String,
    pub service_name: String,
    pub targets: Vec<String>,
    pub permissions: Vec<String>,
    /// Разрешён ли доступ гостям (с временным PoW-токеном через X-Key)
    pub allow_guests: bool,
}

#[derive(sqlx::FromRow)]
struct ServiceRow
{
    id: i64,
    prefix: String,
    service_name: String,
    targets: String,      // JSON
    permissions: String,  // JSON
    allow_guests: bool,
}

impl ServiceRow
{
    fn into_dbo(self) -> Result<ServiceDbo, Error>
    {
        Ok(ServiceDbo
        {
            id: self.id,
            prefix: self.prefix,
            service_name: self.service_name,
            targets: serde_json::from_str(&self.targets)?,
            permissions: serde_json::from_str(&self.permissions)?,
            allow_guests: self.allow_guests,
        })
    }
}

struct ServicesTable;

impl ServicesTable
{
    fn create() -> &'static str
    {
        "CREATE TABLE IF NOT EXISTS services (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            prefix       TEXT NOT NULL UNIQUE,
            service_name TEXT NOT NULL UNIQUE,
            targets      TEXT NOT NULL DEFAULT '[]',
            permissions  TEXT NOT NULL DEFAULT '[]',
            allow_guests INTEGER NOT NULL DEFAULT 0
        )"
    }
}

#[derive(Clone)]
pub struct ServicesRepository
{
    connection: Arc<SqlitePool>,
}

impl ServicesRepository
{
    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self, Error>
    {
        sqlx::query(ServicesTable::create())
            .execute(&*pool).await
            .inspect_err(|e| error!("{}", e))?;

        // Миграция: добавляем колонку, если БД создана до её появления
        let _ = sqlx::query(
            "ALTER TABLE services ADD COLUMN allow_guests INTEGER NOT NULL DEFAULT 0"
        )
        .execute(&*pool).await;

        Ok(Self { connection: pool })
    }

    pub async fn get_all(&self) -> Result<Vec<ServiceDbo>, Error>
    {
        let rows: Vec<ServiceRow> = sqlx::query_as(
            "SELECT id, prefix, service_name, targets, permissions, allow_guests FROM services"
        )
        .fetch_all(&*self.connection)
        .await?;
        rows.into_iter().map(|r| r.into_dbo()).collect()
    }

    pub async fn get_by_prefix(&self, prefix: &str) -> Result<Option<ServiceDbo>, Error>
    {
        let row: Option<ServiceRow> = sqlx::query_as(
            "SELECT id, prefix, service_name, targets, permissions, allow_guests FROM services WHERE prefix = ?"
        )
        .bind(prefix)
        .fetch_optional(&*self.connection)
        .await?;
        row.map(|r| r.into_dbo()).transpose()
    }

    pub async fn create(&self, dbo: &ServiceDbo) -> Result<i64, Error>
    {
        let targets = serde_json::to_string(&dbo.targets)?;
        let permissions = serde_json::to_string(&dbo.permissions)?;
        let id = sqlx::query_scalar(
            "INSERT INTO services (prefix, service_name, targets, permissions, allow_guests)
             VALUES (?, ?, ?, ?, ?)
             RETURNING id"
        )
        .bind(&dbo.prefix)
        .bind(&dbo.service_name)
        .bind(&targets)
        .bind(&permissions)
        .bind(dbo.allow_guests)
        .fetch_one(&*self.connection)
        .await?;
        Ok(id)
    }

    pub async fn update(&self, dbo: &ServiceDbo) -> Result<(), Error>
    {
        let targets = serde_json::to_string(&dbo.targets)?;
        let permissions = serde_json::to_string(&dbo.permissions)?;
        let res = sqlx::query(
            "UPDATE services
             SET prefix = ?, service_name = ?, targets = ?, permissions = ?, allow_guests = ?
             WHERE id = ?"
        )
        .bind(&dbo.prefix)
        .bind(&dbo.service_name)
        .bind(&targets)
        .bind(&permissions)
        .bind(dbo.allow_guests)
        .bind(dbo.id)
        .execute(&*self.connection)
        .await?;
        if res.rows_affected() == 0
        {
            return Err(Error::NotFound(format!("Сервис с id={} не найден", dbo.id)));
        }
        Ok(())
    }

    pub async fn get_guest_services(&self) -> Result<Vec<ServiceDbo>, Error>
    {
        let rows: Vec<ServiceRow> = sqlx::query_as(
            "SELECT id, prefix, service_name, targets, permissions, allow_guests FROM services WHERE allow_guests = 1"
        )
        .fetch_all(&*self.connection)
        .await?;
        rows.into_iter().map(|r| r.into_dbo()).collect()
    }

    pub async fn delete(&self, id: i64) -> Result<(), Error>
    {
        let res = sqlx::query("DELETE FROM services WHERE id = ?")
            .bind(id)
            .execute(&*self.connection)
            .await?;
        if res.rows_affected() == 0
        {
            return Err(Error::NotFound(format!("Сервис с id={} не найден", id)));
        }
        Ok(())
    }
}

use std::sync::Arc;
use sqlx::SqlitePool;
use tracing::error;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct UserDbo
{
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub second_name: String,
    pub surname: String,
    pub avatar: Option<Vec<u8>>,
    pub access: Vec<AccessDbo>,
}

#[derive(Debug, Clone)]
pub struct AccessDbo
{
    pub service_name: String,
    pub permissions: Vec<String>,
}

// --- Промежуточные row-структуры для sqlx (не поддерживает вложенные Vec) ---

#[derive(sqlx::FromRow)]
struct UserRow
{
    id: String,
    username: String,
    password: String,
    first_name: String,
    second_name: String,
    surname: String,
    avatar: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct AccessRow
{
    service_name: String,
    permissions: String, // JSON-массив строк
}

impl UserRow
{
    fn into_dbo(self, access: Vec<AccessDbo>) -> Result<UserDbo, Error>
    {
        let id = uuid::Uuid::parse_str(&self.id)
            .map_err(|e| Error::ArgumentsError(e.to_string()))?;
        Ok(UserDbo
        {
            id,
            username: self.username,
            password: self.password,
            first_name: self.first_name,
            second_name: self.second_name,
            surname: self.surname,
            avatar: self.avatar,
            access,
        })
    }
}

impl AccessRow
{
    fn into_dbo(self) -> Result<AccessDbo, Error>
    {
        let permissions: Vec<String> = serde_json::from_str(&self.permissions)?;
        Ok(AccessDbo { service_name: self.service_name, permissions })
    }
}

// --- DDL ---

struct UsersTable;

impl UsersTable
{
    fn create_users() -> &'static str
    {
        "CREATE TABLE IF NOT EXISTS users (
            id          TEXT NOT NULL PRIMARY KEY,
            username    TEXT NOT NULL UNIQUE,
            password    TEXT NOT NULL,
            first_name  TEXT NOT NULL,
            second_name TEXT NOT NULL,
            surname     TEXT NOT NULL,
            avatar      BLOB
        )"
    }

    fn create_access() -> &'static str
    {
        "CREATE TABLE IF NOT EXISTS user_access (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            service_name TEXT NOT NULL,
            permissions  TEXT NOT NULL DEFAULT '[]',
            UNIQUE(user_id, service_name)
        )"
    }
}

// --- Repository ---

#[derive(Clone)]
pub struct UsersRepository
{
    connection: Arc<SqlitePool>,
}

impl UsersRepository
{
    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self, Error>
    {
        // SQLite не включает FK-каскады по умолчанию
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&*pool).await
            .inspect_err(|e| error!("{}", e))?;

        sqlx::query(UsersTable::create_users())
            .execute(&*pool).await
            .inspect_err(|e| error!("{}", e))?;

        sqlx::query(UsersTable::create_access())
            .execute(&*pool).await
            .inspect_err(|e| error!("{}", e))?;

        Ok(Self { connection: pool })
    }

    pub async fn fetch_access(&self, user_id: &str) -> Result<Vec<AccessDbo>, Error>
    {
        let rows: Vec<AccessRow> = sqlx::query_as(
            "SELECT service_name, permissions FROM user_access WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_all(&*self.connection)
        .await?;
        rows.into_iter().map(|r| r.into_dbo()).collect()
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<UserDbo, Error>
    {
        let id_str = id.to_string();
        let row: UserRow = sqlx::query_as(
            "SELECT id, username, password, first_name, second_name, surname, avatar
             FROM users WHERE id = ?"
        )
        .bind(&id_str)
        .fetch_optional(&*self.connection)
        .await?
        .ok_or(Error::UserNotFound)?;

        let access = self.fetch_access(&id_str).await?;
        row.into_dbo(access)
    }

    pub async fn get_by_username(&self, username: &str) -> Result<UserDbo, Error>
    {
        let row: UserRow = sqlx::query_as(
            "SELECT id, username, password, first_name, second_name, surname, avatar
             FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&*self.connection)
        .await?
        .ok_or(Error::UserNotFound)?;

        let access = self.fetch_access(&row.id).await?;
        row.into_dbo(access)
    }

    pub async fn get_all(&self) -> Result<Vec<UserDbo>, Error>
    {
        let rows: Vec<UserRow> = sqlx::query_as(
            "SELECT id, username, password, first_name, second_name, surname, avatar FROM users"
        )
        .fetch_all(&*self.connection)
        .await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows
        {
            let access = self.fetch_access(&row.id).await?;
            result.push(row.into_dbo(access)?);
        }
        Ok(result)
    }

    pub async fn create(&self, user: &UserDbo) -> Result<(), Error>
    {
        let mut tx = self.connection.begin().await?;

        sqlx::query(
            "INSERT INTO users (id, username, password, first_name, second_name, surname, avatar)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user.id.to_string())
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.first_name)
        .bind(&user.second_name)
        .bind(&user.surname)
        .bind(&user.avatar)
        .execute(&mut *tx)
        .await?;

        for access in &user.access
        {
            let perms = serde_json::to_string(&access.permissions)?;
            sqlx::query(
                "INSERT INTO user_access (user_id, service_name, permissions) VALUES (?, ?, ?)"
            )
            .bind(user.id.to_string())
            .bind(&access.service_name)
            .bind(&perms)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, user: &UserDbo) -> Result<(), Error>
    {
        let id_str = user.id.to_string();
        let mut tx = self.connection.begin().await?;

        let res = sqlx::query(
            "UPDATE users
             SET username = ?, password = ?, first_name = ?, second_name = ?, surname = ?, avatar = ?
             WHERE id = ?"
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.first_name)
        .bind(&user.second_name)
        .bind(&user.surname)
        .bind(&user.avatar)
        .bind(&id_str)
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() == 0
        {
            return Err(Error::UserNotFound);
        }

        // Перезаписываем права: удаляем старые, вставляем новые
        sqlx::query("DELETE FROM user_access WHERE user_id = ?")
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;

        for access in &user.access
        {
            let perms = serde_json::to_string(&access.permissions)?;
            sqlx::query(
                "INSERT INTO user_access (user_id, service_name, permissions) VALUES (?, ?, ?)"
            )
            .bind(&id_str)
            .bind(&access.service_name)
            .bind(&perms)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Удаление пользователя. Права в `user_access` удаляются каскадно (ON DELETE CASCADE).
    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), Error>
    {
        let res = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.connection)
            .await?;

        if res.rows_affected() == 0
        {
            return Err(Error::UserNotFound);
        }
        Ok(())
    }
}

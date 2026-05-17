use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::core::auth::models::{TokenRecord, UserRecord};

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait TokenReadRepository: Send + Sync {
    async fn find_by_sha256<'c, E>(&self, executor: E, token_sha256: &str) -> Result<Option<TokenRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;

    async fn find_by_id<'c, E>(&self, executor: E, token_id: Uuid) -> Result<Option<TokenRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;
}

#[async_trait]
pub trait TokenWriteRepository: Send + Sync {
    async fn create_token(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        token: &TokenRecord,
    ) -> Result<(), RepoError>;

    async fn revoke_token(&self, tx: &mut Transaction<'_, Postgres>, token_id: Uuid) -> Result<(), RepoError>;
}

#[async_trait]
pub trait TokenUpdateRepository: Send + Sync {
    async fn update_last_used_at<'c, E>(
        &self,
        executor: E,
        token_sha256: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<(), RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;
}

#[async_trait]
pub trait UserReadRepository: Send + Sync {
    async fn find_by_id<'c, E>(&self, executor: E, user_id: Uuid) -> Result<Option<UserRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;

    async fn find_by_email<'c, E>(&self, executor: E, email: &str) -> Result<Option<UserRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;
}

#[async_trait]
pub trait UserWriteRepository: Send + Sync {
    async fn create_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
    ) -> Result<UserRecord, RepoError>;
}

#[async_trait]
pub trait BootstrapKeyRepository: Send + Sync {
    async fn claim_bootstrap_key(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        key_hash: &str,
        used_by: Option<Uuid>,
    ) -> Result<bool, RepoError>;
}

#[derive(Debug, Default, Clone)]
pub struct PgTokenRepository;

#[derive(Debug, Default, Clone)]
pub struct PgUserRepository;

#[derive(Debug, Default, Clone)]
pub struct PgBootstrapKeyRepository;

#[async_trait]
impl TokenReadRepository for PgTokenRepository {
    async fn find_by_sha256<'c, E>(&self, executor: E, token_sha256: &str) -> Result<Option<TokenRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let record = sqlx::query_as::<_, TokenRecord>(
            "SELECT id, user_id, label, token_sha256, token_hash, status, last_used_at, created_at \
             FROM personal_access_tokens WHERE token_sha256 = $1",
        )
        .bind(token_sha256)
        .fetch_optional(executor)
        .await?;

        Ok(record)
    }

    async fn find_by_id<'c, E>(&self, executor: E, token_id: Uuid) -> Result<Option<TokenRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let record = sqlx::query_as::<_, TokenRecord>(
            "SELECT id, user_id, label, token_sha256, token_hash, status, last_used_at, created_at \
             FROM personal_access_tokens WHERE id = $1",
        )
        .bind(token_id)
        .fetch_optional(executor)
        .await?;

        Ok(record)
    }
}

#[async_trait]
impl TokenWriteRepository for PgTokenRepository {
    async fn create_token(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        token: &TokenRecord,
    ) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO personal_access_tokens (id, user_id, label, token_sha256, token_hash, status, last_used_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(&token.label)
        .bind(&token.token_sha256)
        .bind(&token.token_hash)
        .bind(token.status.as_str())
        .bind(token.last_used_at)
        .bind(token.created_at)
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    async fn revoke_token(&self, tx: &mut Transaction<'_, Postgres>, token_id: Uuid) -> Result<(), RepoError> {
        sqlx::query("UPDATE personal_access_tokens SET status = 'Revoked' WHERE id = $1")
            .bind(token_id)
            .execute(tx.as_mut())
            .await?;

        Ok(())
    }
}

#[async_trait]
impl TokenUpdateRepository for PgTokenRepository {
    async fn update_last_used_at<'c, E>(
        &self,
        executor: E,
        token_sha256: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<(), RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        sqlx::query("UPDATE personal_access_tokens SET last_used_at = $2 WHERE token_sha256 = $1")
            .bind(token_sha256)
            .bind(timestamp)
            .execute(executor)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl UserReadRepository for PgUserRepository {
    async fn find_by_id<'c, E>(&self, executor: E, user_id: Uuid) -> Result<Option<UserRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT id, email, blocked, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(executor)
        .await?;

        Ok(record)
    }

    async fn find_by_email<'c, E>(&self, executor: E, email: &str) -> Result<Option<UserRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT id, email, blocked, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(executor)
        .await?;

        Ok(record)
    }
}

#[async_trait]
impl UserWriteRepository for PgUserRepository {
    async fn create_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
    ) -> Result<UserRecord, RepoError> {
        let record = sqlx::query_as::<_, UserRecord>(
            "INSERT INTO users (email) VALUES ($1) \
             RETURNING id, email, blocked, created_at, updated_at",
        )
        .bind(email)
        .fetch_one(tx.as_mut())
        .await?;

        Ok(record)
    }
}

#[async_trait]
impl BootstrapKeyRepository for PgBootstrapKeyRepository {
    async fn claim_bootstrap_key(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        key_hash: &str,
        used_by: Option<Uuid>,
    ) -> Result<bool, RepoError> {
        let rows = sqlx::query(
            "INSERT INTO bootstrap_key_usage (key_hash, used_at, used_by) VALUES ($1, NOW(), $2) \
             ON CONFLICT (key_hash) DO NOTHING",
        )
        .bind(key_hash)
        .bind(used_by)
        .execute(tx.as_mut())
        .await?
        .rows_affected();

        Ok(rows == 1)
    }
}

#[derive(Debug, Clone)]
pub struct Repositories {
    pub token: PgTokenRepository,
    pub user: PgUserRepository,
    pub bootstrap: PgBootstrapKeyRepository,
    pub pool: PgPool,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            token: PgTokenRepository,
            user: PgUserRepository,
            bootstrap: PgBootstrapKeyRepository,
            pool,
        }
    }
}

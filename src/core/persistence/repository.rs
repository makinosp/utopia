use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::core::auth::models::{TokenRecord, UserRecord};
use crate::core::compatibility::pagination::Paginated;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait TokenReadRepository: Send + Sync {
    async fn find_by_sha256<'c, E>(
        &self,
        executor: E,
        token_sha256: &str,
    ) -> Result<Option<TokenRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;

    async fn find_by_id<'c, E>(
        &self,
        executor: E,
        token_id: Uuid,
    ) -> Result<Option<TokenRecord>, RepoError>
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

    async fn revoke_token(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        token_id: Uuid,
    ) -> Result<(), RepoError>;
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
    async fn find_by_id<'c, E>(
        &self,
        executor: E,
        user_id: Uuid,
    ) -> Result<Option<UserRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;

    async fn find_by_email<'c, E>(
        &self,
        executor: E,
        email: &str,
    ) -> Result<Option<UserRecord>, RepoError>
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

#[derive(Debug, Clone)]
pub struct AccountListFilter {
    pub page: u32,
    pub limit: u32,
    pub account_type: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountRecord {
    pub id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub account_type: String,
    pub name: String,
    pub current_balance: Decimal,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait AccountReadRepository: Send + Sync {
    async fn list_by_user(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        filter: &AccountListFilter,
    ) -> Result<Paginated<AccountRecord>, RepoError>;
}

#[derive(Debug, Default, Clone)]
pub struct PgTokenRepository;

#[derive(Debug, Default, Clone)]
pub struct PgUserRepository;

#[derive(Debug, Default, Clone)]
pub struct PgBootstrapKeyRepository;

#[derive(Debug, Default, Clone)]
pub struct PgAccountRepository;

#[async_trait]
impl TokenReadRepository for PgTokenRepository {
    async fn find_by_sha256<'c, E>(
        &self,
        executor: E,
        token_sha256: &str,
    ) -> Result<Option<TokenRecord>, RepoError>
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

    async fn find_by_id<'c, E>(
        &self,
        executor: E,
        token_id: Uuid,
    ) -> Result<Option<TokenRecord>, RepoError>
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

    async fn revoke_token(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        token_id: Uuid,
    ) -> Result<(), RepoError> {
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
    async fn find_by_id<'c, E>(
        &self,
        executor: E,
        user_id: Uuid,
    ) -> Result<Option<UserRecord>, RepoError>
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

    async fn find_by_email<'c, E>(
        &self,
        executor: E,
        email: &str,
    ) -> Result<Option<UserRecord>, RepoError>
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

#[async_trait]
impl AccountReadRepository for PgAccountRepository {
    async fn list_by_user(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        filter: &AccountListFilter,
    ) -> Result<Paginated<AccountRecord>, RepoError> {
        let total_records = if let Some(account_type) = &filter.account_type {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM accounts WHERE user_id = $1 AND account_type = $2",
            )
            .bind(user_id)
            .bind(account_type)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM accounts WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(pool)
                .await?
        };

        let offset = i64::from(filter.page.saturating_sub(1)) * i64::from(filter.limit);
        let limit = i64::from(filter.limit);

        let records = if let Some(account_type) = &filter.account_type {
            sqlx::query_as::<_, AccountRecord>(
                "SELECT id, user_id, account_type, name, current_balance, currency_code, created_at, updated_at \
                 FROM accounts \
                 WHERE user_id = $1 AND account_type = $2 \
                 ORDER BY name ASC, id ASC \
                 LIMIT $3 OFFSET $4",
            )
            .bind(user_id)
            .bind(account_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, AccountRecord>(
                "SELECT id, user_id, account_type, name, current_balance, currency_code, created_at, updated_at \
                 FROM accounts \
                 WHERE user_id = $1 \
                 ORDER BY name ASC, id ASC \
                 LIMIT $2 OFFSET $3",
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(Paginated {
            total_records: total_records.max(0) as u64,
            records,
            current_page: u64::from(filter.page),
            per_page: u64::from(filter.limit),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Repositories {
    pub token: PgTokenRepository,
    pub user: PgUserRepository,
    pub bootstrap: PgBootstrapKeyRepository,
    pub account: PgAccountRepository,
    pub pool: PgPool,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            token: PgTokenRepository,
            user: PgUserRepository,
            bootstrap: PgBootstrapKeyRepository,
            account: PgAccountRepository,
            pool,
        }
    }
}

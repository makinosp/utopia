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
    // Extended columns (migration 0003)
    pub active: bool,
    pub initial_balance: Decimal,
    pub initial_balance_date: Option<DateTime<Utc>>,
    pub virtual_balance: Decimal,
    pub deleted_at: Option<DateTime<Utc>>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub account_number: Option<String>,
    pub notes: Option<String>,
    pub include_net_worth: bool,
    pub order: Option<i32>,
    pub account_role: Option<String>,
    pub liability_type: Option<String>,
    pub liability_direction: Option<String>,
    pub interest: Option<String>,
    pub interest_period: Option<String>,
    pub cc_type: Option<String>,
    pub cc_monthly_payment_date: Option<String>,
    pub opening_balance_date: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait AccountReadRepository: Send + Sync {
    async fn list_by_user(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        filter: &AccountListFilter,
    ) -> Result<Paginated<AccountRecord>, RepoError>;

    async fn find_by_id<'c, E>(
        &self,
        executor: E,
        user_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<AccountRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send;
}

/// Column list constant for SELECT queries to avoid duplication
const ACCOUNT_COLUMNS: &str = "\
    id, user_id, account_type, name, \
    current_balance, currency_code, \
    created_at, updated_at, \
    active, initial_balance, initial_balance_date, virtual_balance, \
    deleted_at, iban, bic, account_number, notes, \
    include_net_worth, \"order\", account_role, \
    liability_type, liability_direction, interest, interest_period, \
    cc_type, cc_monthly_payment_date, opening_balance_date";

#[async_trait]
pub trait AccountWriteRepository: Send + Sync {
    async fn create(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        account_type: &str,
        name: &str,
        currency_code: &str,
        initial_balance: Decimal,
        initial_balance_date: Option<DateTime<Utc>>,
        active: bool,
        include_net_worth: bool,
        account_role: Option<&str>,
        iban: Option<&str>,
        bic: Option<&str>,
        account_number: Option<&str>,
        notes: Option<&str>,
        virtual_balance: Decimal,
        liability_type: Option<&str>,
        liability_direction: Option<&str>,
        interest: Option<&str>,
        interest_period: Option<&str>,
        cc_type: Option<&str>,
        cc_monthly_payment_date: Option<&str>,
        opening_balance_date: Option<DateTime<Utc>>,
    ) -> Result<AccountRecord, RepoError>;

    async fn update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        account_type: Option<&str>,
        currency_code: Option<&str>,
        active: Option<bool>,
        include_net_worth: Option<bool>,
        account_role: Option<Option<&str>>,
        iban: Option<Option<&str>>,
        bic: Option<Option<&str>>,
        account_number: Option<Option<&str>>,
        notes: Option<Option<&str>>,
        virtual_balance: Option<Decimal>,
        liability_type: Option<Option<&str>>,
        liability_direction: Option<Option<&str>>,
        interest: Option<Option<&str>>,
        interest_period: Option<Option<&str>>,
        cc_type: Option<Option<&str>>,
        cc_monthly_payment_date: Option<Option<&str>>,
        opening_balance_date: Option<Option<DateTime<Utc>>>,
    ) -> Result<AccountRecord, RepoError>;

    async fn hard_delete(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, RepoError>;
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
            "SELECT id, email, blocked, primary_currency_code, created_at, updated_at FROM users WHERE id = $1",
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
            "SELECT id, email, blocked, primary_currency_code, created_at, updated_at FROM users WHERE email = $1",
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
               RETURNING id, email, blocked, primary_currency_code, created_at, updated_at",
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
                "SELECT COUNT(*) FROM accounts WHERE user_id = $1 AND account_type = $2 AND deleted_at IS NULL",
            )
            .bind(user_id)
            .bind(account_type)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM accounts WHERE user_id = $1 AND deleted_at IS NULL",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await?
        };

        let offset = i64::from(filter.page.saturating_sub(1)) * i64::from(filter.limit);
        let limit = i64::from(filter.limit);

        let records = if let Some(account_type) = &filter.account_type {
            sqlx::query_as::<_, AccountRecord>(&format!(
                "SELECT {} FROM accounts \
                     WHERE user_id = $1 AND account_type = $2 AND deleted_at IS NULL \
                     ORDER BY name ASC, id ASC \
                     LIMIT $3 OFFSET $4",
                ACCOUNT_COLUMNS
            ))
            .bind(user_id)
            .bind(account_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, AccountRecord>(&format!(
                "SELECT {} FROM accounts \
                     WHERE user_id = $1 AND deleted_at IS NULL \
                     ORDER BY name ASC, id ASC \
                     LIMIT $2 OFFSET $3",
                ACCOUNT_COLUMNS
            ))
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

    async fn find_by_id<'c, E>(
        &self,
        executor: E,
        user_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<AccountRecord>, RepoError>
    where
        E: Executor<'c, Database = Postgres> + Send,
    {
        let record = sqlx::query_as::<_, AccountRecord>(&format!(
            "SELECT {} FROM accounts \
                 WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
            ACCOUNT_COLUMNS
        ))
        .bind(account_id)
        .bind(user_id)
        .fetch_optional(executor)
        .await?;

        Ok(record)
    }
}

#[async_trait]
impl AccountWriteRepository for PgAccountRepository {
    #[allow(clippy::too_many_arguments)]
    async fn create(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        account_type: &str,
        name: &str,
        currency_code: &str,
        initial_balance: Decimal,
        initial_balance_date: Option<DateTime<Utc>>,
        active: bool,
        include_net_worth: bool,
        account_role: Option<&str>,
        iban: Option<&str>,
        bic: Option<&str>,
        account_number: Option<&str>,
        notes: Option<&str>,
        virtual_balance: Decimal,
        liability_type: Option<&str>,
        liability_direction: Option<&str>,
        interest: Option<&str>,
        interest_period: Option<&str>,
        cc_type: Option<&str>,
        cc_monthly_payment_date: Option<&str>,
        opening_balance_date: Option<DateTime<Utc>>,
    ) -> Result<AccountRecord, RepoError> {
        let record = sqlx::query_as::<_, AccountRecord>(&format!(
            "INSERT INTO accounts \
                 (user_id, account_type, name, current_balance, currency_code, \
                  active, initial_balance, initial_balance_date, virtual_balance, \
                  iban, bic, account_number, notes, include_net_worth, \"order\", account_role, \
                  liability_type, liability_direction, interest, interest_period, \
                  cc_type, cc_monthly_payment_date, opening_balance_date) \
                 VALUES ($1, $2, $3, $4, $5, \
                         $6, $7, $8, $9, \
                         $10, $11, $12, $13, $14, $15, $16, \
                         $17, $18, $19, $20, \
                         $21, $22, $23) \
                 RETURNING {}",
            ACCOUNT_COLUMNS
        ))
        .bind(user_id)
        .bind(account_type)
        .bind(name)
        .bind(initial_balance) // current_balance = initial_balance on creation
        .bind(currency_code)
        .bind(active)
        .bind(initial_balance)
        .bind(initial_balance_date)
        .bind(virtual_balance)
        .bind(iban)
        .bind(bic)
        .bind(account_number)
        .bind(notes)
        .bind(include_net_worth)
        .bind(None::<i32>) // order: defaults to null
        .bind(account_role)
        .bind(liability_type)
        .bind(liability_direction)
        .bind(interest)
        .bind(interest_period)
        .bind(cc_type)
        .bind(cc_monthly_payment_date)
        .bind(opening_balance_date)
        .fetch_one(tx.as_mut())
        .await?;

        Ok(record)
    }

    #[allow(clippy::too_many_arguments)]
    async fn update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        account_type: Option<&str>,
        currency_code: Option<&str>,
        active: Option<bool>,
        include_net_worth: Option<bool>,
        account_role: Option<Option<&str>>,
        iban: Option<Option<&str>>,
        bic: Option<Option<&str>>,
        account_number: Option<Option<&str>>,
        notes: Option<Option<&str>>,
        virtual_balance: Option<Decimal>,
        liability_type: Option<Option<&str>>,
        liability_direction: Option<Option<&str>>,
        interest: Option<Option<&str>>,
        interest_period: Option<Option<&str>>,
        cc_type: Option<Option<&str>>,
        cc_monthly_payment_date: Option<Option<&str>>,
        opening_balance_date: Option<Option<DateTime<Utc>>>,
    ) -> Result<AccountRecord, RepoError> {
        use sqlx::QueryBuilder;

        let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE accounts SET ");

        let mut sep = builder.separated(", ");

        if let Some(v) = name {
            sep.push("name = ");
            sep.push_bind(v);
        }
        if let Some(v) = account_type {
            sep.push("account_type = ");
            sep.push_bind(v);
        }
        if let Some(v) = currency_code {
            sep.push("currency_code = ");
            sep.push_bind(v);
        }
        if let Some(v) = active {
            sep.push("active = ");
            sep.push_bind(v);
        }
        if let Some(v) = include_net_worth {
            sep.push("include_net_worth = ");
            sep.push_bind(v);
        }
        if let Some(v) = virtual_balance {
            sep.push("virtual_balance = ");
            sep.push_bind(v);
        }

        // Handle Option<Option<T>>: Some(Some(v)) sets the value, Some(None) sets to NULL
        if let Some(ref v) = account_role {
            if let Some(val) = v {
                sep.push("account_role = ");
                sep.push_bind(val);
            } else {
                sep.push("account_role = NULL");
            }
        }
        if let Some(ref v) = iban {
            if let Some(val) = v {
                sep.push("iban = ");
                sep.push_bind(val);
            } else {
                sep.push("iban = NULL");
            }
        }
        if let Some(ref v) = bic {
            if let Some(val) = v {
                sep.push("bic = ");
                sep.push_bind(val);
            } else {
                sep.push("bic = NULL");
            }
        }
        if let Some(ref v) = account_number {
            if let Some(val) = v {
                sep.push("account_number = ");
                sep.push_bind(val);
            } else {
                sep.push("account_number = NULL");
            }
        }
        if let Some(ref v) = notes {
            if let Some(val) = v {
                sep.push("notes = ");
                sep.push_bind(val);
            } else {
                sep.push("notes = NULL");
            }
        }
        if let Some(ref v) = liability_type {
            if let Some(val) = v {
                sep.push("liability_type = ");
                sep.push_bind(val);
            } else {
                sep.push("liability_type = NULL");
            }
        }
        if let Some(ref v) = liability_direction {
            if let Some(val) = v {
                sep.push("liability_direction = ");
                sep.push_bind(val);
            } else {
                sep.push("liability_direction = NULL");
            }
        }
        if let Some(ref v) = interest {
            if let Some(val) = v {
                sep.push("interest = ");
                sep.push_bind(val);
            } else {
                sep.push("interest = NULL");
            }
        }
        if let Some(ref v) = interest_period {
            if let Some(val) = v {
                sep.push("interest_period = ");
                sep.push_bind(val);
            } else {
                sep.push("interest_period = NULL");
            }
        }
        if let Some(ref v) = cc_type {
            if let Some(val) = v {
                sep.push("cc_type = ");
                sep.push_bind(val);
            } else {
                sep.push("cc_type = NULL");
            }
        }
        if let Some(ref v) = cc_monthly_payment_date {
            if let Some(val) = v {
                sep.push("cc_monthly_payment_date = ");
                sep.push_bind(val);
            } else {
                sep.push("cc_monthly_payment_date = NULL");
            }
        }
        if let Some(ref v) = opening_balance_date {
            if let Some(val) = v {
                sep.push("opening_balance_date = ");
                sep.push_bind(val);
            } else {
                sep.push("opening_balance_date = NULL");
            }
        }

        // If no fields to update, return error (nothing to update)
        if builder.sql().ends_with("SET ") {
            return Err(RepoError::Database(sqlx::Error::RowNotFound));
        }

        builder.push(" WHERE id = ");
        builder.push_bind(account_id);
        builder.push(" AND user_id = ");
        builder.push_bind(user_id);
        builder.push(" AND deleted_at IS NULL RETURNING ");
        builder.push(ACCOUNT_COLUMNS);

        let record = builder
            .build_query_as::<AccountRecord>()
            .fetch_optional(tx.as_mut())
            .await?;

        record.ok_or(RepoError::Database(sqlx::Error::RowNotFound))
    }

    async fn hard_delete(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        account_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, RepoError> {
        let rows = sqlx::query("DELETE FROM accounts WHERE id = $1 AND user_id = $2")
            .bind(account_id)
            .bind(user_id)
            .execute(tx.as_mut())
            .await?
            .rows_affected();

        Ok(rows > 0)
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

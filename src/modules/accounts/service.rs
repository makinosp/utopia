use std::collections::HashMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::auth::models::Principal;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::DomainError;
use crate::core::persistence::repository::{
    AccountReadRepository, AccountWriteRepository, PgAccountRepository,
};

use super::types::{
    normalize_account_type_create, validation_error, AccountListQuery, AccountView,
    CreateAccountRequest, UpdateAccountRequest,
};

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Application service for account operations.
///
/// Encapsulates business rules, ownership checks, and transaction boundaries
/// for all account-related use cases.
#[async_trait]
pub trait AccountService: Send + Sync {
    /// List accounts for the authenticated user with optional type filter and pagination.
    async fn list_accounts(
        &self,
        query: AccountListQuery,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<Paginated<AccountView>, DomainError>;

    /// Get a single account by ID (ownership enforced by repository).
    async fn get_account(
        &self,
        account_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError>;

    /// Create a new account with optional opening balance.
    async fn create_account(
        &self,
        req: CreateAccountRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError>;

    /// Update an existing account.
    async fn update_account(
        &self,
        account_id: Uuid,
        req: UpdateAccountRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError>;

    /// Delete an account.
    async fn delete_account(
        &self,
        account_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<(), DomainError>;
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AccountServiceImpl {
    read_repo: PgAccountRepository,
    write_repo: PgAccountRepository,
}

impl AccountServiceImpl {
    pub fn new(repository: PgAccountRepository) -> Self {
        Self {
            read_repo: repository.clone(),
            write_repo: repository,
        }
    }
}

#[async_trait]
impl AccountService for AccountServiceImpl {
    async fn list_accounts(
        &self,
        query: AccountListQuery,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<Paginated<AccountView>, DomainError> {
        let filter = crate::core::persistence::repository::AccountListFilter {
            page: query.page,
            limit: query.limit,
            account_type: query.account_type,
        };
        let accounts = self
            .read_repo
            .list_by_user(pool, principal.user_id, &filter)
            .await
            .map_err(|_| DomainError::Persistence)?;
        Ok(Paginated {
            total_records: accounts.total_records,
            records: accounts
                .records
                .into_iter()
                .map(AccountView::from)
                .collect(),
            current_page: accounts.current_page,
            per_page: accounts.per_page,
        })
    }

    async fn get_account(
        &self,
        account_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError> {
        self.read_repo
            .find_by_id(pool, principal.user_id, account_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .map(AccountView::from)
            .ok_or(DomainError::NotFound)
    }

    async fn create_account(
        &self,
        req: CreateAccountRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError> {
        let normalized_type = normalize_account_type_create(&req.account_type)?;
        let currency_code = req.currency_code.ok_or_else(|| {
            validation_error("currency_code", "The currency code field is required.")
        })?;
        let opening_balance = req.opening_balance.unwrap_or(Decimal::ZERO);
        let opening_balance_date = req.opening_balance_date;
        let virtual_balance = req.virtual_balance.unwrap_or(Decimal::ZERO);

        // Asset accounts require account_role
        if normalized_type == "asset" && req.account_role.is_none() {
            let mut fields = HashMap::new();
            fields.insert(
                "account_role".to_string(),
                vec!["The account role field is required for asset accounts.".to_string()],
            );
            return Err(DomainError::Validation(fields));
        }

        let mut tx = pool.begin().await.map_err(|_| DomainError::Persistence)?;

        let record = self
            .write_repo
            .create(
                &mut tx,
                principal.user_id,
                &normalized_type,
                &req.name,
                &currency_code,
                opening_balance,
                opening_balance_date,
                req.active.unwrap_or(true),
                req.include_net_worth.unwrap_or(true),
                req.account_role.as_deref(),
                req.iban.as_deref(),
                req.bic.as_deref(),
                req.account_number.as_deref(),
                req.notes.as_deref(),
                virtual_balance,
                req.liability_type.as_deref(),
                req.liability_direction.as_deref(),
                req.interest.as_deref(),
                req.interest_period.as_deref(),
                None, // cc_type
                None, // cc_monthly_payment_date
                opening_balance_date,
            )
            .await
            .map_err(|e| map_repo_error(&e))?;

        tx.commit().await.map_err(|_| DomainError::Persistence)?;
        Ok(AccountView::from(record))
    }

    async fn update_account(
        &self,
        account_id: Uuid,
        req: UpdateAccountRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError> {
        // Verify ownership first
        self.read_repo
            .find_by_id(pool, principal.user_id, account_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .ok_or(DomainError::NotFound)?;

        let mut tx = pool.begin().await.map_err(|_| DomainError::Persistence)?;

        let record = self
            .write_repo
            .update(
                &mut tx,
                account_id,
                principal.user_id,
                req.name.as_deref(),
                req.account_type.as_deref(),
                req.currency_code.as_deref(),
                req.active,
                req.include_net_worth,
                req.account_role.as_ref().map(|v| v.as_deref()),
                req.iban.as_ref().map(|v| v.as_deref()),
                req.bic.as_ref().map(|v| v.as_deref()),
                req.account_number.as_ref().map(|v| v.as_deref()),
                req.notes.as_ref().map(|v| v.as_deref()),
                req.virtual_balance,
                req.liability_type.as_ref().map(|v| v.as_deref()),
                req.liability_direction.as_ref().map(|v| v.as_deref()),
                req.interest.as_ref().map(|v| v.as_deref()),
                req.interest_period.as_ref().map(|v| v.as_deref()),
                None, // cc_type
                None, // cc_monthly_payment_date
                req.opening_balance_date.as_ref().map(|v| *v),
            )
            .await
            .map_err(|e| map_repo_error(&e))?;

        tx.commit().await.map_err(|_| DomainError::Persistence)?;
        Ok(AccountView::from(record))
    }

    async fn delete_account(
        &self,
        account_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<(), DomainError> {
        // Verify ownership first
        self.read_repo
            .find_by_id(pool, principal.user_id, account_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .ok_or(DomainError::NotFound)?;

        let mut tx = pool.begin().await.map_err(|_| DomainError::Persistence)?;

        let deleted = self
            .write_repo
            .hard_delete(&mut tx, account_id, principal.user_id)
            .await
            .map_err(|_| DomainError::Persistence)?;

        if !deleted {
            return Err(DomainError::NotFound);
        }

        tx.commit().await.map_err(|_| DomainError::Persistence)?;
        Ok(())
    }
}

fn map_repo_error(err: &crate::core::persistence::repository::RepoError) -> DomainError {
    match err {
        crate::core::persistence::repository::RepoError::Database(e) => {
            if let Some(pg_err) = e.as_database_error() {
                if let Some(code) = pg_err.code() {
                    // 23505 = unique_violation (duplicate name)
                    if code.as_ref() == "23505" {
                        let mut fields = HashMap::new();
                        fields.insert(
                            "name".to_string(),
                            vec!["The name has already been taken.".to_string()],
                        );
                        return DomainError::Validation(fields);
                    }
                }
            }
            DomainError::Persistence
        }
    }
}

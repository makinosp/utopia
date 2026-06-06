use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::core::auth::models::Principal;
use crate::core::compatibility::decimal_amount::DecimalAmount;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::DomainError;
use crate::core::persistence::repository::{
    AccountBalanceUpdate, AccountReadRepository, PgAccountRepository, PgTransactionRepository,
    RepoError, TransactionFilter, TransactionReadRepository, TransactionRecord,
    TransactionWriteRepository,
};

pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_LIMIT: u32 = 50;
pub const MAX_LIMIT: u32 = 100;

const ALLOWED_TRANSACTION_TYPES: &[&str] = &["withdrawal", "deposit", "transfer"];

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionListQuery {
    pub page: u32,
    pub limit: u32,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub transaction_type: Option<String>,
}

impl TransactionListQuery {
    pub fn from_params(
        page: Option<&str>,
        limit: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        transaction_type: Option<&str>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            page: parse_page(page)?,
            limit: parse_limit(limit)?,
            start_date: parse_optional_datetime(start_date, "start")?,
            end_date: parse_optional_datetime(end_date, "end")?,
            transaction_type: normalize_transaction_type(transaction_type)?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransactionRequest {
    pub group_id: Option<Uuid>,
    pub transaction_type: String,
    pub description: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub date: Option<DateTime<Utc>>,
    pub source_id: Option<Uuid>,
    pub destination_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub notes: Option<String>,
    pub reconciled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTransactionRequest {
    pub transaction_type: Option<String>,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
    #[allow(dead_code)]
    pub currency_code: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub source_id: Option<Option<Uuid>>,
    pub destination_id: Option<Option<Uuid>>,
    pub category_name: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub reconciled: Option<bool>,
}

// ---------------------------------------------------------------------------
// View model (domain -> DTO)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TransactionView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub transaction_type: String,
    pub description: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub date: DateTime<Utc>,
    pub source_id: Option<Uuid>,
    pub destination_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub notes: Option<String>,
    pub reconciled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Resolved names (joined at query time or separately)
    pub source_name: Option<String>,
    pub destination_name: Option<String>,
}

impl From<TransactionRecord> for TransactionView {
    fn from(r: TransactionRecord) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            group_id: r.group_id,
            transaction_type: r.transaction_type,
            description: r.description,
            amount: r.amount,
            currency_code: r.currency_code,
            date: r.date,
            source_id: r.source_id,
            destination_id: r.destination_id,
            category_name: r.category_name,
            notes: r.notes,
            reconciled: r.reconciled,
            created_at: r.created_at,
            updated_at: r.updated_at,
            source_name: None,
            destination_name: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Firefly-III compatible DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct FireflyTransactionAttributes {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user: String,
    pub group_id: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub date: DateTime<Utc>,
    pub description: String,
    pub amount: DecimalAmount,
    pub currency_code: String,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub destination_id: Option<String>,
    pub destination_name: Option<String>,
    pub category_name: Option<String>,
    pub notes: Option<String>,
    pub reconciled: bool,
}

impl Default for FireflyTransactionAttributes {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user: String::new(),
            group_id: String::new(),
            transaction_type: String::new(),
            date: Utc::now(),
            description: String::new(),
            amount: DecimalAmount(Decimal::ZERO),
            currency_code: String::new(),
            source_id: None,
            source_name: None,
            destination_id: None,
            destination_name: None,
            category_name: None,
            notes: None,
            reconciled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyLink {
    pub rel: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyTransactionResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
    pub attributes: FireflyTransactionAttributes,
    pub links: Vec<FireflyLink>,
}

impl FireflyTransactionResource {
    pub fn from_view(view: TransactionView) -> Self {
        Self {
            resource_type: "transactions".to_string(),
            id: view.id.to_string(),
            attributes: FireflyTransactionAttributes {
                created_at: view.created_at,
                updated_at: view.updated_at,
                user: String::new(), // Resolved by handler from principal
                group_id: view.group_id.to_string(),
                transaction_type: view.transaction_type,
                date: view.date,
                description: view.description,
                amount: DecimalAmount(view.amount),
                currency_code: view.currency_code,
                source_id: view.source_id.map(|id| id.to_string()),
                source_name: view.source_name,
                destination_id: view.destination_id.map(|id| id.to_string()),
                destination_name: view.destination_name,
                category_name: view.category_name,
                notes: view.notes,
                reconciled: view.reconciled,
            },
            links: vec![FireflyLink {
                rel: "self".to_string(),
                uri: format!("/api/v1/transactions/{}", view.id),
            }],
        }
    }
}

impl From<TransactionView> for FireflyTransactionResource {
    fn from(view: TransactionView) -> Self {
        Self::from_view(view)
    }
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

fn parse_page(raw: Option<&str>) -> Result<u32, DomainError> {
    parse_positive_u32(
        raw,
        DEFAULT_PAGE,
        "page",
        "The page field must be at least 1.",
    )
}

fn parse_limit(raw: Option<&str>) -> Result<u32, DomainError> {
    let limit = parse_positive_u32(
        raw,
        DEFAULT_LIMIT,
        "limit",
        "The limit field must be at least 1.",
    )?;
    if limit > MAX_LIMIT {
        return Err(validation_error(
            "limit",
            "The limit field may not be greater than 100.",
        ));
    }
    Ok(limit)
}

fn parse_positive_u32(
    raw: Option<&str>,
    default_value: u32,
    field: &str,
    min_message: &str,
) -> Result<u32, DomainError> {
    let Some(raw) = raw else {
        return Ok(default_value);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(validation_error(field, min_message));
    }
    let parsed = trimmed
        .parse::<u32>()
        .map_err(|_| validation_error(field, &format!("The {field} field must be an integer.")))?;
    if parsed == 0 {
        return Err(validation_error(field, min_message));
    }
    Ok(parsed)
}

fn parse_optional_datetime(
    raw: Option<&str>,
    field: &str,
) -> Result<Option<DateTime<Utc>>, DomainError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let dt = chrono::DateTime::parse_from_rfc3339(trimmed).map_err(|_| {
        validation_error(
            field,
            &format!("The {field} field must be a valid ISO 8601 date."),
        )
    })?;
    Ok(Some(dt.with_timezone(&Utc)))
}

fn normalize_transaction_type(raw: Option<&str>) -> Result<Option<String>, DomainError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "all" {
        return Ok(None);
    }
    if !ALLOWED_TRANSACTION_TYPES.contains(&normalized.as_str()) {
        return Err(validation_error("type", "The selected type is invalid."));
    }
    Ok(Some(normalized))
}

fn validate_transaction_type(raw: &str) -> Result<String, DomainError> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() || !ALLOWED_TRANSACTION_TYPES.contains(&normalized.as_str()) {
        return Err(validation_error(
            "transaction_type",
            "The selected type is invalid.",
        ));
    }
    Ok(normalized)
}

fn validation_error(field: &str, message: &str) -> DomainError {
    let mut fields = HashMap::new();
    fields.insert(field.to_string(), vec![message.to_string()]);
    DomainError::Validation(fields)
}

/// Determine the balance impact of a transaction on its source and destination accounts.
fn balance_impacts(
    transaction_type: &str,
    amount: Decimal,
    source_id: Option<Uuid>,
    destination_id: Option<Uuid>,
) -> Vec<AccountBalanceUpdate> {
    match transaction_type {
        "withdrawal" => {
            let mut updates = Vec::new();
            if let Some(src) = source_id {
                updates.push(AccountBalanceUpdate {
                    account_id: src,
                    delta: -amount,
                });
            }
            updates
        }
        "deposit" => {
            let mut updates = Vec::new();
            if let Some(dst) = destination_id {
                updates.push(AccountBalanceUpdate {
                    account_id: dst,
                    delta: amount,
                });
            }
            updates
        }
        "transfer" => {
            let mut updates = Vec::new();
            if let Some(src) = source_id {
                updates.push(AccountBalanceUpdate {
                    account_id: src,
                    delta: -amount,
                });
            }
            if let Some(dst) = destination_id {
                updates.push(AccountBalanceUpdate {
                    account_id: dst,
                    delta: amount,
                });
            }
            updates
        }
        _ => vec![],
    }
}

/// Reverse the balance impact (used for delete/update).
fn reverse_balance_impacts(
    transaction_type: &str,
    amount: Decimal,
    source_id: Option<Uuid>,
    destination_id: Option<Uuid>,
) -> Vec<AccountBalanceUpdate> {
    balance_impacts(transaction_type, amount, source_id, destination_id)
        .into_iter()
        .map(|u| AccountBalanceUpdate {
            account_id: u.account_id,
            delta: -u.delta,
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct TransactionService {
    read_repo: PgTransactionRepository,
    write_repo: PgTransactionRepository,
    account_read_repo: PgAccountRepository,
}

impl TransactionService {
    pub fn new(
        transaction_repo: PgTransactionRepository,
        account_repo: PgAccountRepository,
    ) -> Self {
        Self {
            read_repo: transaction_repo.clone(),
            write_repo: transaction_repo,
            account_read_repo: account_repo,
        }
    }

    /// Resolve account names for a list of transaction views.
    async fn resolve_account_names(
        &self,
        views: &mut [TransactionView],
        pool: &PgPool,
    ) -> Result<(), DomainError> {
        // Collect all account IDs that need to be resolved
        let mut account_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        for view in views.iter() {
            if let Some(src_id) = view.source_id {
                account_ids.insert(src_id);
            }
            if let Some(dst_id) = view.destination_id {
                account_ids.insert(dst_id);
            }
        }

        // If no account IDs to resolve, return early
        if account_ids.is_empty() {
            return Ok(());
        }

        // Fetch all account names in a single query
        let account_records = self
            .account_read_repo
            .find_by_ids(
                pool,
                views[0].user_id,
                &account_ids.into_iter().collect::<Vec<_>>(),
            )
            .await
            .map_err(|_| DomainError::Persistence)?;

        // Create a map for quick lookup
        let account_map: std::collections::HashMap<Uuid, String> = account_records
            .into_iter()
            .map(|record| (record.id, record.name))
            .collect();

        // Populate the source_name and destination_name fields
        for view in views.iter_mut() {
            if let Some(src_id) = view.source_id {
                if let Some(name) = account_map.get(&src_id) {
                    view.source_name = Some(name.clone());
                }
            }
            if let Some(dst_id) = view.destination_id {
                if let Some(name) = account_map.get(&dst_id) {
                    view.destination_name = Some(name.clone());
                }
            }
        }

        Ok(())
    }

    /// List transactions for the authenticated user with optional filters.
    pub async fn list_transactions(
        &self,
        query: TransactionListQuery,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<Paginated<TransactionView>, DomainError> {
        let filter = TransactionFilter {
            page: query.page,
            limit: query.limit,
            start_date: query.start_date,
            end_date: query.end_date,
            transaction_type: query.transaction_type.clone(),
        };

        let result = self
            .read_repo
            .list_by_user(pool, principal.user_id, &filter)
            .await
            .map_err(|_| DomainError::Persistence)?;

        let total_records = result.total_records;
        let mut records: Vec<TransactionView> = result
            .records
            .into_iter()
            .map(TransactionView::from)
            .collect();

        self.resolve_account_names(&mut records, pool).await?;

        Ok(Paginated {
            total_records,
            records,
            current_page: u64::from(query.page),
            per_page: u64::from(query.limit),
        })
    }

    /// Get a single transaction by ID (ownership enforced).
    pub async fn get_transaction(
        &self,
        transaction_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<TransactionView, DomainError> {
        let mut view: TransactionView = self
            .read_repo
            .find_by_id(pool, principal.user_id, transaction_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .map(TransactionView::from)
            .ok_or(DomainError::NotFound)?;

        self.resolve_account_names(std::slice::from_mut(&mut view), pool)
            .await?;

        Ok(view)
    }

    /// Validate that source/destination accounts exist and belong to the user,
    /// and lock them with SELECT FOR UPDATE for concurrency-safe balance updates.
    /// This must be called within an active DB transaction.
    async fn validate_and_lock_accounts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        principal: &Principal,
        transaction_type: &str,
        source_id: Option<Uuid>,
        destination_id: Option<Uuid>,
    ) -> Result<(), DomainError> {
        match transaction_type {
            "withdrawal" => {
                if source_id.is_none() {
                    return Err(validation_error(
                        "source_id",
                        "The source account is required for withdrawals.",
                    ));
                }
            }
            "deposit" => {
                if destination_id.is_none() {
                    return Err(validation_error(
                        "destination_id",
                        "The destination account is required for deposits.",
                    ));
                }
            }
            "transfer" => {
                if source_id.is_none() {
                    return Err(validation_error(
                        "source_id",
                        "The source account is required for transfers.",
                    ));
                }
                if destination_id.is_none() {
                    return Err(validation_error(
                        "destination_id",
                        "The destination account is required for transfers.",
                    ));
                }
                if source_id == destination_id {
                    return Err(validation_error(
                        "destination_id",
                        "Source and destination must be different for transfers.",
                    ));
                }
            }
            _ => {}
        }

        // Collect unique account IDs for locking
        let mut lock_ids: Vec<Uuid> = Vec::new();
        let mut src_id: Option<Uuid> = None;
        if let Some(src) = source_id {
            lock_ids.push(src);
            src_id = Some(src);
        }
        if let Some(dst) = destination_id {
            if src_id != Some(dst) {
                lock_ids.push(dst);
            }
        }

        // Sort and deduplicate IDs to ensure consistent locking order and prevent deadlocks
        let mut lock_ids: Vec<Uuid> = lock_ids.into_iter().collect();
        lock_ids.sort();
        lock_ids.dedup();

        if lock_ids.is_empty() {
            return Ok(());
        }

        // Lock and validate in a single operation
        let locked = self
            .account_read_repo
            .lock_accounts_for_update(tx, principal.user_id, &lock_ids)
            .await
            .map_err(|e| {
                // Detect deadlock (PostgreSQL error 40P01)
                let RepoError::Database(ref dbe) = e;
                if let Some(pg_err) = dbe.as_database_error() {
                    if let Some(code) = pg_err.code() {
                        if code.as_ref() == "40P01" {
                            return DomainError::Conflict(
                                "A concurrent modification was detected. Please retry.".to_string(),
                            );
                        }
                    }
                }
                DomainError::Persistence
            })?;

        // Verify all requested accounts were found
        let locked_ids: Vec<Uuid> = locked.iter().map(|r| r.id).collect();
        if let Some(src) = source_id {
            if !locked_ids.contains(&src) {
                return Err(validation_error(
                    "source_id",
                    "The selected source account is invalid.",
                ));
            }
        }
        if let Some(dst) = destination_id {
            if !locked_ids.contains(&dst) {
                return Err(validation_error(
                    "destination_id",
                    "The selected destination account is invalid.",
                ));
            }
        }

        Ok(())
    }

    /// Create a new transaction with atomic balance updates.
    pub async fn create_transaction(
        &self,
        req: CreateTransactionRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<TransactionView, DomainError> {
        let transaction_type = validate_transaction_type(&req.transaction_type)?;

        if req.description.trim().is_empty() {
            return Err(validation_error(
                "description",
                "The description field is required.",
            ));
        }

        if req.description.len() > 255 {
            return Err(validation_error(
                "description",
                "The description field must not exceed 255 characters.",
            ));
        }

        if req.amount <= Decimal::ZERO {
            return Err(validation_error(
                "amount",
                "The amount must be a positive number.",
            ));
        }

        let group_id = req.group_id.unwrap_or_else(Uuid::new_v4);
        let date = req.date.unwrap_or_else(Utc::now);

        let mut tx = pool.begin().await.map_err(|_| DomainError::Persistence)?;

        // Validate accounts and lock them with SELECT FOR UPDATE
        self.validate_and_lock_accounts(
            &mut tx,
            principal,
            &transaction_type,
            req.source_id,
            req.destination_id,
        )
        .await?;

        let record = self
            .write_repo
            .create(
                &mut tx,
                crate::core::persistence::repository::CreateTransactionRequest {
                    user_id: principal.user_id,
                    group_id,
                    transaction_type: transaction_type.clone(),
                    description: req.description.clone(),
                    amount: req.amount,
                    currency_code: req.currency_code.clone(),
                    date,
                    source_id: req.source_id,
                    destination_id: req.destination_id,
                    category_name: req.category_name.clone(),
                    notes: req.notes.clone(),
                    reconciled: req.reconciled.unwrap_or(false),
                },
            )
            .await
            .map_err(|_| DomainError::Persistence)?;

        // Update account balances atomically
        let balance_updates = balance_impacts(
            &transaction_type,
            req.amount,
            req.source_id,
            req.destination_id,
        );
        if !balance_updates.is_empty() {
            self.write_repo
                .update_account_balances(&mut tx, &balance_updates)
                .await
                .map_err(|_| DomainError::Persistence)?;
        }

        tx.commit().await.map_err(|_| DomainError::Persistence)?;

        let mut view = TransactionView::from(record);
        self.resolve_account_names(std::slice::from_mut(&mut view), pool)
            .await?;

        Ok(view)
    }

    /// Update an existing transaction with atomic balance adjustments.
    pub async fn update_transaction(
        &self,
        transaction_id: Uuid,
        req: UpdateTransactionRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<TransactionView, DomainError> {
        // Fetch original to compute balance delta
        let original: TransactionRecord = self
            .read_repo
            .find_by_id(pool, principal.user_id, transaction_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .ok_or(DomainError::NotFound)?;

        // Validate type if provided
        if let Some(ref t) = req.transaction_type {
            validate_transaction_type(t)?;
        }

        let new_type = req
            .transaction_type
            .as_deref()
            .unwrap_or(&original.transaction_type);
        let new_amount = req.amount.unwrap_or(original.amount);
        let new_source = req.source_id.unwrap_or(original.source_id);
        let new_dest = req.destination_id.unwrap_or(original.destination_id);

        // Validate amount if provided
        if let Some(amount) = req.amount {
            if amount <= Decimal::ZERO {
                return Err(validation_error(
                    "amount",
                    "The amount must be a positive number.",
                ));
            }
        }

        // Validate description length if provided
        if let Some(ref desc) = req.description {
            if desc.len() > 255 {
                return Err(validation_error(
                    "description",
                    "The description field must not exceed 255 characters.",
                ));
            }
        }

        // Re-validate accounts with new values and lock them
        let mut tx = pool.begin().await.map_err(|_| DomainError::Persistence)?;

        self.validate_and_lock_accounts(&mut tx, principal, new_type, new_source, new_dest)
            .await?;

        let result = self
            .write_repo
            .update(
                &mut tx,
                transaction_id,
                principal.user_id,
                crate::core::persistence::repository::UpdateTransactionRequest {
                    description: req.description.clone(),
                    amount: req.amount,
                    date: req.date,
                    source_id: req.source_id,
                    destination_id: req.destination_id,
                    category_name: req.category_name.clone(),
                    notes: req.notes.clone(),
                    reconciled: req.reconciled,
                },
            )
            .await
            .map_err(|_| DomainError::Persistence)?;

        // Reverse old balance impacts, apply new ones
        let old_impacts = reverse_balance_impacts(
            &original.transaction_type,
            original.amount,
            original.source_id,
            original.destination_id,
        );
        if !old_impacts.is_empty() {
            self.write_repo
                .update_account_balances(&mut tx, &old_impacts)
                .await
                .map_err(|_| DomainError::Persistence)?;
        }

        let new_impacts = balance_impacts(new_type, new_amount, new_source, new_dest);
        if !new_impacts.is_empty() {
            self.write_repo
                .update_account_balances(&mut tx, &new_impacts)
                .await
                .map_err(|_| DomainError::Persistence)?;
        }

        tx.commit().await.map_err(|_| DomainError::Persistence)?;

        let mut view = TransactionView::from(result);
        self.resolve_account_names(std::slice::from_mut(&mut view), pool)
            .await?;

        Ok(view)
    }

    /// Delete a transaction with atomic balance reversal.
    pub async fn delete_transaction(
        &self,
        transaction_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<(), DomainError> {
        let original: TransactionRecord = self
            .read_repo
            .find_by_id(pool, principal.user_id, transaction_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .ok_or(DomainError::NotFound)?;

        let mut tx = pool.begin().await.map_err(|_| DomainError::Persistence)?;

        // Lock affected accounts before reversing balances
        let mut lock_ids: Vec<Uuid> = Vec::new();
        if let Some(src) = original.source_id {
            lock_ids.push(src);
        }
        if let Some(dst) = original.destination_id {
            if !lock_ids.contains(&dst) {
                lock_ids.push(dst);
            }
        }

        // Sort and deduplicate IDs to ensure consistent locking order and prevent deadlocks
        lock_ids.sort();
        lock_ids.dedup();

        if !lock_ids.is_empty() {
            self.account_read_repo
                .lock_accounts_for_update(&mut tx, principal.user_id, &lock_ids)
                .await
                .map_err(|_| DomainError::Persistence)?;
        }

        let deleted = self
            .write_repo
            .hard_delete(&mut tx, transaction_id, principal.user_id)
            .await
            .map_err(|_| DomainError::Persistence)?;

        if deleted.is_none() {
            return Err(DomainError::NotFound);
        }

        // Reverse balance impacts
        let balance_updates = reverse_balance_impacts(
            &original.transaction_type,
            original.amount,
            original.source_id,
            original.destination_id,
        );
        if !balance_updates.is_empty() {
            self.write_repo
                .update_account_balances(&mut tx, &balance_updates)
                .await
                .map_err(|_| DomainError::Persistence)?;
        }

        tx.commit().await.map_err(|_| DomainError::Persistence)?;

        Ok(())
    }

    /// List transactions for a specific account.
    pub async fn list_account_transactions(
        &self,
        account_id: Uuid,
        principal: &Principal,
        pool: &PgPool,
        page: u32,
        limit: u32,
    ) -> Result<Paginated<TransactionView>, DomainError> {
        // Verify account exists and belongs to user
        self.account_read_repo
            .find_by_id(pool, principal.user_id, account_id)
            .await
            .map_err(|_| DomainError::Persistence)?
            .ok_or(DomainError::NotFound)?;

        let result = self
            .read_repo
            .list_by_account(pool, principal.user_id, account_id, page, limit)
            .await
            .map_err(|_| DomainError::Persistence)?;

        let total_records = result.total_records;
        let mut records: Vec<TransactionView> = result
            .records
            .into_iter()
            .map(TransactionView::from)
            .collect();

        self.resolve_account_names(&mut records, pool).await?;

        Ok(Paginated {
            total_records,
            records,
            current_page: u64::from(page),
            per_page: u64::from(limit),
        })
    }
}

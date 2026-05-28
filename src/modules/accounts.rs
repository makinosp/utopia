use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::auth::models::Principal;
use crate::core::compatibility::decimal_amount::DecimalAmount;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::DomainError;
use crate::core::persistence::repository::{
    AccountListFilter, AccountReadRepository, AccountRecord, AccountWriteRepository,
    PgAccountRepository,
};

pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_LIMIT: u32 = 50;
pub const MAX_LIMIT: u32 = 100;

const ALLOWED_ACCOUNT_TYPES: &[&str] = &[
    "asset",
    "cash",
    "expense",
    "revenue",
    "special",
    "hidden",
    "liability",
    "liabilities",
    "credit card",
    "default account",
    "cash account",
    "asset account",
    "expense account",
    "revenue account",
    "initial balance account",
    "beneficiary account",
    "import account",
    "reconciliation account",
    "loan",
    "debt",
    "mortgage",
];

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountListQuery {
    pub page: u32,
    pub limit: u32,
    pub account_type: Option<String>,
}

impl AccountListQuery {
    pub fn from_params(
        page: Option<&str>,
        limit: Option<&str>,
        account_type: Option<&str>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            page: parse_page(page)?,
            limit: parse_limit(limit)?,
            account_type: normalize_account_type(account_type)?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency_code: Option<String>,
    pub active: Option<bool>,
    pub include_net_worth: Option<bool>,
    pub account_role: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub account_number: Option<String>,
    pub opening_balance: Option<Decimal>,
    pub opening_balance_date: Option<DateTime<Utc>>,
    pub virtual_balance: Option<Decimal>,
    pub notes: Option<String>,
    pub liability_type: Option<String>,
    pub liability_direction: Option<String>,
    pub interest: Option<String>,
    pub interest_period: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub currency_code: Option<String>,
    pub active: Option<bool>,
    pub include_net_worth: Option<bool>,
    pub account_role: Option<Option<String>>,
    pub iban: Option<Option<String>>,
    pub bic: Option<Option<String>>,
    pub account_number: Option<Option<String>>,
    pub opening_balance: Option<Option<Decimal>>,
    pub opening_balance_date: Option<Option<DateTime<Utc>>>,
    pub virtual_balance: Option<Decimal>,
    pub notes: Option<Option<String>>,
    pub liability_type: Option<Option<String>>,
    pub liability_direction: Option<Option<String>>,
    pub interest: Option<Option<String>>,
    pub interest_period: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// View model (domain → DTO)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AccountView {
    pub id: Uuid,
    pub account_type: String,
    pub name: String,
    pub current_balance: Decimal,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Extended attributes
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

impl From<AccountRecord> for AccountView {
    fn from(r: AccountRecord) -> Self {
        Self {
            id: r.id,
            account_type: r.account_type,
            name: r.name,
            current_balance: r.current_balance,
            currency_code: r.currency_code,
            created_at: r.created_at,
            updated_at: r.updated_at,
            active: r.active,
            initial_balance: r.initial_balance,
            initial_balance_date: r.initial_balance_date,
            virtual_balance: r.virtual_balance,
            deleted_at: r.deleted_at,
            iban: r.iban,
            bic: r.bic,
            account_number: r.account_number,
            notes: r.notes,
            include_net_worth: r.include_net_worth,
            order: r.order,
            account_role: r.account_role,
            liability_type: r.liability_type,
            liability_direction: r.liability_direction,
            interest: r.interest,
            interest_period: r.interest_period,
            cc_type: r.cc_type,
            cc_monthly_payment_date: r.cc_monthly_payment_date,
            opening_balance_date: r.opening_balance_date,
        }
    }
}

// ---------------------------------------------------------------------------
// Firefly-III compatible DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct FireflyAccountAttributes {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency_code: String,
    pub current_balance: DecimalAmount,
    // Phase 1 extended attributes
    pub active: bool,
    pub order: Option<i32>,
    pub currency_name: Option<String>,
    pub currency_symbol: Option<String>,
    pub currency_decimal_places: i32,
    pub primary_currency_code: String,
    pub primary_currency_name: String,
    pub primary_currency_symbol: String,
    pub primary_currency_decimal_places: i32,
    pub initial_balance: Option<DecimalAmount>,
    pub virtual_balance: Option<DecimalAmount>,
    pub current_balance_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub account_number: Option<String>,
    pub opening_balance: Option<DecimalAmount>,
    pub opening_balance_date: Option<DateTime<Utc>>,
    pub include_net_worth: bool,
    pub account_role: Option<String>,
    pub liability_type: Option<String>,
    pub liability_direction: Option<String>,
    pub interest: Option<String>,
    pub interest_period: Option<String>,
    pub credit_card_type: Option<String>,
    pub monthly_payment_date: Option<String>,
}

impl Default for FireflyAccountAttributes {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            name: String::new(),
            account_type: String::new(),
            currency_code: String::new(),
            current_balance: DecimalAmount(Decimal::ZERO),
            active: true,
            order: None,
            currency_name: None,
            currency_symbol: None,
            currency_decimal_places: 2,
            primary_currency_code: String::new(),
            primary_currency_name: String::new(),
            primary_currency_symbol: String::new(),
            primary_currency_decimal_places: 2,
            initial_balance: None,
            virtual_balance: None,
            current_balance_date: Utc::now(),
            notes: None,
            iban: None,
            bic: None,
            account_number: None,
            opening_balance: None,
            opening_balance_date: None,
            include_net_worth: true,
            account_role: None,
            liability_type: None,
            liability_direction: None,
            interest: None,
            interest_period: None,
            credit_card_type: None,
            monthly_payment_date: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyLink {
    pub rel: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyAccountResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
    pub attributes: FireflyAccountAttributes,
    pub links: Vec<FireflyLink>,
}

impl FireflyAccountResource {
    pub fn from_view(view: AccountView, primary_currency_code: &str) -> Self {
        let currency_symbol = currency_symbol_from_code(&view.currency_code);
        let currency_name = currency_name_from_code(&view.currency_code);
        let primary_currency_symbol = currency_symbol_from_code(primary_currency_code);
        let primary_currency_name = currency_name_from_code(primary_currency_code);

        Self {
            resource_type: "accounts".to_string(),
            id: view.id.to_string(),
            attributes: FireflyAccountAttributes {
                created_at: view.created_at,
                updated_at: view.updated_at,
                name: view.name,
                account_type: view.account_type,
                currency_code: view.currency_code.clone(),
                current_balance: DecimalAmount(view.current_balance),
                active: view.active,
                order: view.order,
                currency_name: Some(currency_name),
                currency_symbol: Some(currency_symbol),
                currency_decimal_places: 2,
                primary_currency_code: primary_currency_code.to_string(),
                primary_currency_name,
                primary_currency_symbol,
                primary_currency_decimal_places: 2,
                initial_balance: Some(DecimalAmount(view.initial_balance)),
                virtual_balance: Some(DecimalAmount(view.virtual_balance)),
                current_balance_date: view.updated_at,
                notes: view.notes,
                iban: view.iban,
                bic: view.bic,
                account_number: view.account_number,
                opening_balance: Some(DecimalAmount(view.initial_balance)),
                opening_balance_date: view.initial_balance_date,
                include_net_worth: view.include_net_worth,
                account_role: view.account_role,
                liability_type: view.liability_type,
                liability_direction: view.liability_direction,
                interest: view.interest,
                interest_period: view.interest_period,
                credit_card_type: view.cc_type,
                monthly_payment_date: view.cc_monthly_payment_date,
            },
            links: vec![FireflyLink {
                rel: "self".to_string(),
                uri: format!("/api/v1/accounts/{}", view.id),
            }],
        }
    }
}

impl From<AccountView> for FireflyAccountResource {
    fn from(view: AccountView) -> Self {
        Self::from_view(view, "JPY")
    }
}

// ---------------------------------------------------------------------------
// Currency helpers
// ---------------------------------------------------------------------------

const CURRENCY_TABLE: &[(&str, &str, &str)] = &[
    ("JPY", "Japanese Yen", "¥"),
    ("USD", "US Dollar", "$"),
    ("EUR", "Euro", "€"),
    ("GBP", "British Pound", "£"),
    ("CHF", "Swiss Franc", "Fr"),
    ("CAD", "Canadian Dollar", "C$"),
    ("AUD", "Australian Dollar", "A$"),
    ("CNY", "Chinese Yuan", "¥"),
    ("KRW", "South Korean Won", "₩"),
    ("INR", "Indian Rupee", "₹"),
    ("BRL", "Brazilian Real", "R$"),
    ("SEK", "Swedish Krona", "kr"),
    ("NOK", "Norwegian Krone", "kr"),
    ("DKK", "Danish Krone", "kr"),
    ("NZD", "New Zealand Dollar", "NZ$"),
    ("SGD", "Singapore Dollar", "S$"),
    ("HKD", "Hong Kong Dollar", "HK$"),
    ("TRY", "Turkish Lira", "₺"),
    ("MXN", "Mexican Peso", "Mex$"),
    ("ZAR", "South African Rand", "R"),
];

fn currency_symbol_from_code(code: &str) -> String {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, _, s)| s.to_string())
        .unwrap_or_else(|| code.to_string())
}

fn currency_name_from_code(code: &str) -> String {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, n, _)| n.to_string())
        .unwrap_or_else(|| code.to_string())
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

fn normalize_account_type(raw: Option<&str>) -> Result<Option<String>, DomainError> {
    let Some(raw) = raw else {
        return Ok(None);
    };

    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(validation_error("type", "The selected type is invalid."));
    }

    if normalized == "all" {
        return Ok(None);
    }

    if !ALLOWED_ACCOUNT_TYPES.contains(&normalized.as_str()) {
        return Err(validation_error("type", "The selected type is invalid."));
    }

    Ok(Some(normalized))
}

fn normalize_account_type_create(raw: &str) -> Result<String, DomainError> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() || !ALLOWED_ACCOUNT_TYPES.contains(&normalized.as_str()) {
        return Err(validation_error("type", "The selected type is invalid."));
    }
    Ok(normalized)
}

fn validation_error(field: &str, message: &str) -> DomainError {
    let mut fields = HashMap::new();
    fields.insert(field.to_string(), vec![message.to_string()]);
    DomainError::Validation(fields)
}

#[derive(Debug, Clone)]
pub struct AccountService {
    read_repo: PgAccountRepository,
    write_repo: PgAccountRepository,
}

impl AccountService {
    pub fn new(repository: PgAccountRepository) -> Self {
        Self {
            read_repo: repository.clone(),
            write_repo: repository,
        }
    }

    /// List accounts for the authenticated user with optional type filter and pagination.
    pub async fn list_accounts(
        &self,
        query: AccountListQuery,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<Paginated<AccountView>, DomainError> {
        let filter = AccountListFilter {
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

    /// Get a single account by ID (ownership enforced by repository).
    pub async fn get_account(
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

    /// Create a new account with optional opening balance.
    pub async fn create_account(
        &self,
        req: CreateAccountRequest,
        principal: &Principal,
        pool: &PgPool,
    ) -> Result<AccountView, DomainError> {
        let normalized_type = normalize_account_type_create(&req.account_type)?;
        let currency_code = req.currency_code.unwrap_or_else(|| "JPY".to_string());
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

    /// Update an existing account.
    pub async fn update_account(
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

    /// Soft-delete an account (hide from API, preserve data).
    pub async fn delete_account(
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
            .soft_delete(&mut tx, account_id, principal.user_id)
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

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::auth::models::Principal;
use crate::core::compatibility::decimal_amount::DecimalAmount;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::DomainError;
use crate::core::persistence::repository::{
    AccountListFilter, AccountReadRepository, AccountRecord, PgAccountRepository,
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

#[derive(Debug, Clone)]
pub struct AccountView {
    pub id: Uuid,
    pub account_type: String,
    pub name: String,
    pub current_balance: Decimal,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AccountRecord> for AccountView {
    fn from(record: AccountRecord) -> Self {
        Self {
            id: record.id,
            account_type: record.account_type,
            name: record.name,
            current_balance: record.current_balance,
            currency_code: record.currency_code,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyAccountAttributes {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: String,
    pub currency_code: String,
    pub current_balance: DecimalAmount,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyAccountResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
    pub attributes: FireflyAccountAttributes,
}

impl From<AccountView> for FireflyAccountResource {
    fn from(view: AccountView) -> Self {
        Self {
            resource_type: "accounts".to_string(),
            id: view.id.to_string(),
            attributes: FireflyAccountAttributes {
                created_at: view.created_at,
                updated_at: view.updated_at,
                name: view.name,
                account_type: view.account_type,
                currency_code: view.currency_code,
                current_balance: DecimalAmount(view.current_balance),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountService {
    repository: PgAccountRepository,
}

impl AccountService {
    pub fn new(repository: PgAccountRepository) -> Self {
        Self { repository }
    }

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
            .repository
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
}

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

fn validation_error(field: &str, message: &str) -> DomainError {
    let mut fields = HashMap::new();
    fields.insert(field.to_string(), vec![message.to_string()]);
    DomainError::Validation(fields)
}

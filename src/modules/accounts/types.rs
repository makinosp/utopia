use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::compatibility::decimal_amount::DecimalAmount;
use crate::core::compatibility::pagination::{DEFAULT_LIMIT, DEFAULT_PAGE, MAX_LIMIT};
use crate::core::error_mapping::mapper::DomainError;
use crate::core::persistence::repository::AccountRecord;

// Basic account types (core types) - documented for reference
#[allow(dead_code)]
const BASIC_ACCOUNT_TYPES: &[&str] = &[
    "asset",
    "cash",
    "expense",
    "revenue",
    "special",
    "hidden",
    "liability",
];

// Extended account types (Firefly-III specific variations) - documented for reference
#[allow(dead_code)]
const EXTENDED_ACCOUNT_TYPES: &[&str] = &[
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        let currency_decimal_places = currency_decimal_places_from_code(&view.currency_code);
        let primary_currency_symbol = currency_symbol_from_code(primary_currency_code);
        let primary_currency_name = currency_name_from_code(primary_currency_code);
        let primary_currency_decimal_places =
            currency_decimal_places_from_code(primary_currency_code);

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
                currency_decimal_places: currency_decimal_places as i32,
                primary_currency_code: primary_currency_code.to_string(),
                primary_currency_name,
                primary_currency_symbol,
                primary_currency_decimal_places: primary_currency_decimal_places as i32,
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

const CURRENCY_TABLE: &[(&str, &str, &str, u32)] = &[
    ("JPY", "Japanese Yen", "¥", 0),
    ("USD", "US Dollar", "$", 2),
    ("EUR", "Euro", "€", 2),
    ("GBP", "British Pound", "£", 2),
    ("CHF", "Swiss Franc", "Fr", 2),
    ("CAD", "Canadian Dollar", "C$", 2),
    ("AUD", "Australian Dollar", "A$", 2),
    ("CNY", "Chinese Yuan", "¥", 2),
    ("KRW", "South Korean Won", "₩", 0),
    ("INR", "Indian Rupee", "₹", 2),
    ("BRL", "Brazilian Real", "R$", 2),
    ("SEK", "Swedish Krona", "kr", 2),
    ("NOK", "Norwegian Krone", "kr", 2),
    ("DKK", "Danish Krone", "kr", 2),
    ("NZD", "New Zealand Dollar", "NZ$", 2),
    ("SGD", "Singapore Dollar", "S$", 2),
    ("HKD", "Hong Kong Dollar", "HK$", 2),
    ("TRY", "Turkish Lira", "₺", 2),
    ("MXN", "Mexican Peso", "Mex$", 2),
    ("ZAR", "South African Rand", "R", 2),
];

fn currency_symbol_from_code(code: &str) -> String {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _, _)| *c == code)
        .map(|(_, _, s, _)| s.to_string())
        .unwrap_or_else(|| code.to_string())
}

fn currency_name_from_code(code: &str) -> String {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _, _)| *c == code)
        .map(|(_, n, _, _)| n.to_string())
        .unwrap_or_else(|| code.to_string())
}

fn currency_decimal_places_from_code(code: &str) -> u32 {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _, _)| *c == code)
        .map(|(_, _, _, d)| *d)
        .unwrap_or(2)
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

pub fn parse_page(raw: Option<&str>) -> Result<u32, DomainError> {
    parse_positive_u32(
        raw,
        DEFAULT_PAGE,
        "page",
        "The page field must be at least 1.",
    )
}

pub fn parse_limit(raw: Option<&str>) -> Result<u32, DomainError> {
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

pub fn normalize_account_type(raw: Option<&str>) -> Result<Option<String>, DomainError> {
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

pub fn normalize_account_type_create(raw: &str) -> Result<String, DomainError> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() || !ALLOWED_ACCOUNT_TYPES.contains(&normalized.as_str()) {
        return Err(validation_error("type", "The selected type is invalid."));
    }
    Ok(normalized)
}

pub fn validation_error(field: &str, message: &str) -> DomainError {
    let mut fields = HashMap::new();
    fields.insert(field.to_string(), vec![message.to_string()]);
    DomainError::Validation(fields)
}

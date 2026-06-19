use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::auth::models::UserRecord;
use crate::core::compatibility::envelope::FireflyListEnvelope;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::DomainError;

// ---------------------------------------------------------------------------
// Currency data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CurrencyEntry {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub decimal_places: u32,
    pub default: bool,
    pub enabled: bool,
}

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

fn all_currencies() -> Vec<CurrencyEntry> {
    CURRENCY_TABLE
        .iter()
        .enumerate()
        .map(|(i, (code, name, symbol, decimals))| CurrencyEntry {
            code: code.to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimal_places: *decimals,
            default: i == 0,
            enabled: true,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Firefly-III compatible DTOs — Currency
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct FireflyCurrencyAttributes {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub decimal_places: u32,
    pub default: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyCurrencyLink {
    pub rel: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyCurrencyResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
    pub attributes: FireflyCurrencyAttributes,
    pub links: Vec<FireflyCurrencyLink>,
}

impl From<CurrencyEntry> for FireflyCurrencyResource {
    fn from(entry: CurrencyEntry) -> Self {
        let now = Utc::now();
        Self {
            resource_type: "currencies".to_string(),
            id: entry.code.clone(),
            attributes: FireflyCurrencyAttributes {
                created_at: now,
                updated_at: now,
                code: entry.code.clone(),
                name: entry.name.clone(),
                symbol: entry.symbol.clone(),
                decimal_places: entry.decimal_places,
                default: entry.default,
                enabled: entry.enabled,
            },
            links: vec![FireflyCurrencyLink {
                rel: "self".to_string(),
                uri: format!("/api/v1/currencies/{}", entry.code),
            }],
        }
    }
}

// ---------------------------------------------------------------------------
// Firefly-III compatible DTOs — User (/about/user)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct FireflyUserAttributes {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub blocked: bool,
    pub blocked_code: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyUserResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
    pub attributes: FireflyUserAttributes,
}

impl From<UserRecord> for FireflyUserResource {
    fn from(user: UserRecord) -> Self {
        Self {
            resource_type: "users".to_string(),
            id: user.id.to_string(),
            attributes: FireflyUserAttributes {
                created_at: user.created_at,
                updated_at: user.updated_at,
                email: user.email,
                blocked: user.blocked,
                blocked_code: None,
                role: "owner".to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Firefly-III compatible DTOs — System info (/about)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct FireflySystemInfoAttributes {
    pub title: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflySystemInfoResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: u32,
    pub attributes: FireflySystemInfoAttributes,
}

impl FireflySystemInfoResource {
    pub fn new(id: u32, title: &str, value: &str) -> Self {
        Self {
            resource_type: "system_info".to_string(),
            id,
            attributes: FireflySystemInfoAttributes {
                title: title.to_string(),
                value: value.to_string(),
            },
        }
    }
}

fn system_info_entries() -> Vec<FireflySystemInfoResource> {
    vec![
        FireflySystemInfoResource::new(1, "version", env!("CARGO_PKG_VERSION")),
        FireflySystemInfoResource::new(2, "api_version", "1.1.0"),
        FireflySystemInfoResource::new(3, "php_version", "Rust"),
        FireflySystemInfoResource::new(4, "os", std::env::consts::OS),
        FireflySystemInfoResource::new(5, "driver", "PostgreSQL"),
    ]
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MetadataService;

impl MetadataService {
    /// List all currencies with pagination.
    pub fn list_currencies(page: u32, limit: u32) -> FireflyListEnvelope<FireflyCurrencyResource> {
        let all: Vec<FireflyCurrencyResource> =
            all_currencies().into_iter().map(From::from).collect();

        let total = all.len() as u64;

        // Apply pagination
        let offset = ((page.max(1) - 1) as usize).min(all.len());
        let end = (offset + limit as usize).min(all.len());
        let page_records = if offset < all.len() {
            all[offset..end].to_vec()
        } else {
            vec![]
        };

        let paginated = Paginated {
            total_records: total,
            records: page_records,
            current_page: page as u64,
            per_page: limit as u64,
        };

        FireflyListEnvelope::from_paginated(paginated)
    }

    /// Get the authenticated user's profile.
    pub async fn get_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<FireflyUserResource, DomainError> {
        let user = sqlx::query_as::<_, UserRecord>(
            "SELECT id, email, blocked, primary_currency_code, created_at, updated_at \
             FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| DomainError::Persistence)?
        .ok_or(DomainError::NotFound)?;

        Ok(FireflyUserResource::from(user))
    }

    /// Get system information.
    pub fn get_system_info() -> FireflyListEnvelope<FireflySystemInfoResource> {
        let records = system_info_entries();
        let total = records.len() as u64;

        let paginated = Paginated {
            total_records: total,
            records,
            current_page: 1,
            per_page: total.max(1),
        };

        FireflyListEnvelope::from_paginated(paginated)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers (shared with accounts module)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn currency_symbol_from_code(code: &str) -> String {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _, _)| *c == code)
        .map(|(_, _, s, _)| s.to_string())
        .unwrap_or_else(|| code.to_string())
}

#[allow(dead_code)]
pub fn currency_name_from_code(code: &str) -> String {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _, _)| *c == code)
        .map(|(_, n, _, _)| n.to_string())
        .unwrap_or_else(|| code.to_string())
}

#[allow(dead_code)]
pub fn currency_decimal_places(code: &str) -> u32 {
    CURRENCY_TABLE
        .iter()
        .find(|(c, _, _, _)| *c == code)
        .map(|(_, _, _, d)| *d)
        .unwrap_or(2)
}

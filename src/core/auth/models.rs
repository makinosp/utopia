use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub user_id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub blocked: bool,
    pub primary_currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TokenRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    pub token_sha256: String,
    pub token_hash: String,
    pub status: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TokenRecord {
    pub fn is_revoked(&self) -> bool {
        self.status == "Revoked"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIssuancePayload {
    pub id: Uuid,
    pub label: String,
    pub token: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIssuanceResponse {
    pub data: TokenIssuancePayload,
}

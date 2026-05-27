use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use uuid::Uuid;

use crate::core::auth::error::AuthError;

#[derive(Debug, Clone)]
pub struct TokenCacheEntry {
    pub user_id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone)]
pub enum CachedAuthResult {
    Valid(TokenCacheEntry),
    Invalid { reason: AuthError },
}

#[derive(Debug, Clone)]
pub struct TokenCache {
    positive: Arc<Cache<String, TokenCacheEntry>>,
    negative: Arc<Cache<String, AuthError>>,
}

impl TokenCache {
    pub fn new(ttl_secs: u64, negative_ttl_secs: u64, max_capacity: u64) -> Self {
        let positive = Cache::builder()
            .time_to_live(Duration::from_secs(ttl_secs))
            .max_capacity(max_capacity)
            .build();

        let negative = Cache::builder()
            .time_to_live(Duration::from_secs(negative_ttl_secs))
            .max_capacity(max_capacity)
            .build();

        Self {
            positive: Arc::new(positive),
            negative: Arc::new(negative),
        }
    }

    pub async fn get(&self, sha256_token: &str) -> Option<CachedAuthResult> {
        if let Some(entry) = self.positive.get(sha256_token).await {
            return Some(CachedAuthResult::Valid(entry));
        }

        if let Some(reason) = self.negative.get(sha256_token).await {
            return Some(CachedAuthResult::Invalid { reason });
        }

        None
    }

    pub async fn insert_valid(&self, sha256_token: String, entry: TokenCacheEntry) {
        self.positive.insert(sha256_token, entry).await;
    }

    pub async fn insert_invalid(&self, sha256_token: String, error: AuthError) {
        self.negative.insert(sha256_token, error).await;
    }

    pub async fn invalidate(&self, sha256_token: &str) {
        self.positive.invalidate(sha256_token).await;
        self.negative.invalidate(sha256_token).await;
    }
}

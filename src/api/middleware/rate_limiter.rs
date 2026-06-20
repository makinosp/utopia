use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header::RETRY_AFTER, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokio::sync::RwLock;

use crate::app::AppState;
use crate::core::auth::audit_logger::AuditLogger;
use crate::core::auth::error::AuthError;
use crate::core::compatibility::error_response::FireflyErrorResponse;

#[derive(Debug, Clone)]
struct RateLimitEntry {
    window_start: Instant,
    count: u64,
}

#[derive(Debug, Clone)]
pub struct RateLimitState {
    inner: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
    max_requests: u64,
    window_secs: u64,
}

impl RateLimitState {
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    pub async fn check_and_count(&self, ip: IpAddr) -> Result<(), AuthError> {
        let now = Instant::now();
        let window_duration = std::time::Duration::from_secs(self.window_secs);

        // Fast path: read lock to check without blocking concurrent requests
        {
            let map = self.inner.read().await;
            if let Some(entry) = map.get(&ip) {
                if now.duration_since(entry.window_start) <= window_duration {
                    // Window is still active
                    if entry.count > self.max_requests {
                        return Err(AuthError::RateLimitExceeded {
                            retry_after_secs: self.window_secs,
                        });
                    }
                    // Under limit but need to increment — fall through to write path
                }
                // Window expired — fall through to write path for reset
            }
            // IP not seen before — fall through to write path
        }

        // Slow path: write lock for mutation
        let mut map = self.inner.write().await;
        let entry = map.entry(ip).or_insert(RateLimitEntry {
            window_start: now,
            count: 0,
        });

        if now.duration_since(entry.window_start) > window_duration {
            // Window expired: reset
            entry.window_start = now;
            entry.count = 1;
            return Ok(());
        }

        entry.count = entry.count.saturating_add(1);
        if entry.count > self.max_requests {
            Err(AuthError::RateLimitExceeded {
                retry_after_secs: self.window_secs,
            })
        } else {
            Ok(())
        }
    }

    pub fn run_eviction_task(self: Arc<Self>) {
        let evict_interval = std::time::Duration::from_secs(60);
        let max_age = std::time::Duration::from_secs(self.window_secs * 2);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(evict_interval).await;
                let mut map = self.inner.write().await;
                let now = Instant::now();
                map.retain(|_, entry| now.duration_since(entry.window_start) <= max_age);
            }
        });
    }
}

/// Fail-open wrapper: if the rate limit check encounters non-rate-limit errors
/// (e.g., lock contention, overflow), the request is allowed through and a
/// warning is logged.
async fn fail_open_check_and_count(state: &AppState, ip: IpAddr) -> Result<(), AuthError> {
    match state.rate_limit_state.check_and_count(ip).await {
        Ok(()) => Ok(()),
        Err(err @ AuthError::RateLimitExceeded { .. }) => Err(err),
        Err(other) => {
            tracing::warn!(
                target: "rate_limiter",
                error = %other,
                "rate limit check failed, allowing request through"
            );
            Ok(())
        }
    }
}

pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_client_ip(request.headers())
        .unwrap_or_else(|| "127.0.0.1".parse::<IpAddr>().expect("valid fallback ip"));

    let result = fail_open_check_and_count(&state, ip).await;

    match result {
        Ok(()) => next.run(request).await,
        Err(err @ AuthError::RateLimitExceeded { retry_after_secs }) => {
            // Increment rate limit metric
            state
                .metrics
                .rate_limited_requests_total
                .with_label_values(&["bootstrap_tokens", "window_exceeded"])
                .inc();

            // Emit audit log event with source_ip and context fields
            let mut event = AuditLogger::new_event(
                "rate_limit",
                "denied",
                None,
                Some(ip.to_string()),
                Some(err.reason_code()),
                request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string()),
            );
            event.context.insert(
                "window_limit".to_string(),
                state.config.bootstrap_rate_limit_requests.to_string(),
            );
            event.context.insert(
                "window_secs".to_string(),
                state.config.bootstrap_rate_limit_window_secs.to_string(),
            );
            state.audit_logger.emit(event);

            let body = FireflyErrorResponse {
                message: format!(
                    "Too many requests. Please retry after {} seconds.",
                    retry_after_secs
                ),
                errors: HashMap::new(),
            };

            let response = (StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
            let mut response = response;
            response.headers_mut().insert(
                RETRY_AFTER,
                retry_after_secs.to_string().parse().expect("valid header"),
            );
            response
        }
        Err(_) => {
            // Other auth errors - forward to next handler which will handle properly
            next.run(request).await
        }
    }
}

fn extract_client_ip(headers: &axum::http::HeaderMap) -> Option<IpAddr> {
    // Try X-Forwarded-For first, then X-Real-IP, then connection info fallback
    if let Some(value) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next().map(|s| s.trim()))
    {
        if let Ok(ip) = value.parse::<IpAddr>() {
            return Some(ip);
        }
    }

    if let Some(value) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        if let Ok(ip) = value.parse::<IpAddr>() {
            return Some(ip);
        }
    }

    None
}

use std::sync::Arc;

use anyhow::Context;

use crate::api::router::build_router;
use crate::config::AppConfig;
use crate::core::auth::audit_logger::AuditLogger;
use crate::core::auth::cache::TokenCache;
use crate::core::auth::metrics::PrometheusMetrics;
use crate::core::auth::service::TokenService;
use crate::core::persistence::db::create_pool;
use crate::core::persistence::repository::Repositories;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub config: AppConfig,
    pub repositories: Repositories,
    pub cache: TokenCache,
    pub metrics: Arc<PrometheusMetrics>,
    pub audit_logger: AuditLogger,
    pub token_service: TokenService,
}

pub async fn build_app(config: AppConfig) -> anyhow::Result<axum::Router> {
    let pool = create_pool(&config)
        .await
        .context("failed to create pg pool")?;
    let repositories = Repositories::new(pool.clone());
    let metrics = Arc::new(PrometheusMetrics::new());
    let cache = TokenCache::new(
        config.token_cache_ttl_secs,
        config.negative_token_cache_ttl_secs,
        config.token_cache_max_capacity,
    );
    let audit_logger = AuditLogger;
    let token_service = TokenService::new(
        config.clone(),
        repositories.token.clone(),
        repositories.user.clone(),
        repositories.bootstrap.clone(),
        Arc::clone(&metrics),
    );

    let state = Arc::new(AppState {
        config,
        repositories,
        cache,
        metrics: Arc::clone(&metrics),
        audit_logger,
        token_service,
    });

    Ok(build_router(state))
}

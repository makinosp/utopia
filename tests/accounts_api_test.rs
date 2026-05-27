use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;
use utopia::api::router::build_router;
use utopia::app::AppState;
use utopia::config::AppConfig;
use utopia::core::auth::audit_logger::AuditLogger;
use utopia::core::auth::cache::TokenCache;
use utopia::core::auth::metrics::PrometheusMetrics;
use utopia::core::auth::models::{Principal, UserRecord};
use utopia::core::auth::service::TokenService;
use utopia::core::persistence::repository::Repositories;

#[tokio::test]
async fn returns_401_when_bearer_token_is_missing() {
    let database_url =
        "postgres://postgres:postgres@localhost/utopia_test?sslmode=disable".to_string();
    let pool = PgPoolOptions::new()
        .connect_lazy(&database_url)
        .expect("lazy pool");

    let app = build_test_router(pool, test_config(database_url));
    let request = Request::builder()
        .uri("/api/v1/accounts")
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let payload: Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(payload["errors"], json!({}));
    assert!(payload["message"]
        .as_str()
        .expect("message")
        .contains("unauthenticated"));
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn lists_accounts_in_firefly_format_with_pagination_and_type_filter() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "maya@example.com").await;
    let other_user = create_user(&test_db.pool, "other@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("integration".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Alpha Asset",
        Decimal::new(125050, 2),
        "JPY",
    )
    .await;
    seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Zulu Asset",
        Decimal::new(4510, 2),
        "JPY",
    )
    .await;
    seed_account(
        &test_db.pool,
        user.id,
        "expense",
        "Groceries",
        Decimal::new(0, 0),
        "JPY",
    )
    .await;
    seed_account(
        &test_db.pool,
        other_user.id,
        "asset",
        "Other User Account",
        Decimal::new(9999, 2),
        "JPY",
    )
    .await;

    let request = Request::builder()
        .uri("/api/v1/accounts?type=asset&page=1&limit=1")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let payload: Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(payload["meta"]["pagination"]["total"], json!(2));
    assert_eq!(payload["meta"]["pagination"]["count"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["per_page"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["current_page"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["total_pages"], json!(2));

    let account = &payload["data"][0];
    assert_eq!(account["type"], json!("accounts"));
    assert_eq!(account["attributes"]["type"], json!("asset"));
    assert_eq!(account["attributes"]["name"], json!("Alpha Asset"));
    assert_eq!(account["attributes"]["current_balance"], json!("1250.50"));
    assert_eq!(account["attributes"]["currency_code"], json!("JPY"));
}

fn test_config(database_url: String) -> AppConfig {
    AppConfig {
        database_url,
        argon2_memory_cost: 65_536,
        argon2_time_cost: 3,
        argon2_parallelism: 1,
        token_cache_ttl_secs: 60,
        negative_token_cache_ttl_secs: 60,
        token_cache_max_capacity: 100,
        app_port: 3000,
        log_level: "info".to_string(),
        bootstrap_key: "bootstrap-test-key-1234".to_string(),
        bootstrap_user_email: "bootstrap@example.com".to_string(),
    }
}

fn build_test_router(pool: sqlx::PgPool, config: AppConfig) -> axum::Router {
    let state = build_test_state(pool, config);
    build_router(state)
}

fn build_test_state(pool: sqlx::PgPool, config: AppConfig) -> Arc<AppState> {
    let repositories = Repositories::new(pool.clone());
    let metrics = Arc::new(PrometheusMetrics::new());
    let cache = TokenCache::new(60, 60, 100);
    let token_service = TokenService::new(
        config.clone(),
        repositories.token.clone(),
        repositories.user.clone(),
        repositories.bootstrap.clone(),
        Arc::clone(&metrics),
    );

    Arc::new(AppState {
        config,
        repositories,
        cache,
        metrics,
        audit_logger: AuditLogger,
        token_service,
    })
}

struct TestDatabase {
    _container: ContainerAsync<Postgres>,
    database_url: String,
    pool: sqlx::PgPool,
}

async fn start_postgres() -> TestDatabase {
    let container = Postgres::default()
        .with_tag("17-alpine")
        .start()
        .await
        .expect("start postgres");

    let host = container.get_host().await.expect("host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres port");
    let database_url =
        format!("postgres://postgres:postgres@{host}:{port}/postgres?sslmode=disable");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("connect pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");

    TestDatabase {
        _container: container,
        database_url,
        pool,
    }
}

async fn create_user(pool: &sqlx::PgPool, email: &str) -> UserRecord {
    sqlx::query_as::<_, UserRecord>(
        "INSERT INTO users (email) VALUES ($1) RETURNING id, email, blocked, created_at, updated_at",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("create user")
}

async fn seed_account(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    account_type: &str,
    name: &str,
    current_balance: Decimal,
    currency_code: &str,
) {
    sqlx::query(
        "INSERT INTO accounts (user_id, account_type, name, current_balance, currency_code) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(account_type)
    .bind(name)
    .bind(current_balance)
    .bind(currency_code)
    .execute(pool)
    .await
    .expect("insert account");
}

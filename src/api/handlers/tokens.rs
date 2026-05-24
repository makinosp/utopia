use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header::HeaderName, HeaderMap, StatusCode};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::core::auth::error::AuthError;
use crate::core::auth::models::Principal;
use crate::core::error_mapping::mapper::map_auth_error;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub label: String,
}

pub async fn issue_token_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Json(request): Json<TokenRequest>,
) -> Result<
    Json<crate::core::auth::models::TokenIssuanceResponse>,
    (
        StatusCode,
        Json<crate::core::compatibility::error_response::FireflyErrorResponse>,
    ),
> {
    let response = state
        .token_service
        .issue_token(request.label, &principal, &state.repositories.pool)
        .await
        .map_err(map_auth_error_to_json)?;

    Ok(Json(response))
}

pub async fn bootstrap_issue_token_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<TokenRequest>,
) -> Result<
    Json<crate::core::auth::models::TokenIssuanceResponse>,
    (
        StatusCode,
        Json<crate::core::compatibility::error_response::FireflyErrorResponse>,
    ),
> {
    let bootstrap_key = headers
        .get(HeaderName::from_static("x-bootstrap-key"))
        .and_then(|value| value.to_str().ok());

    let response = state
        .token_service
        .issue_bootstrap_token(request.label, bootstrap_key, &state.repositories.pool)
        .await
        .map_err(map_auth_error_to_json)?;

    Ok(Json(response))
}

pub async fn revoke_token_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Path(token_id): Path<Uuid>,
) -> Result<
    StatusCode,
    (
        StatusCode,
        Json<crate::core::compatibility::error_response::FireflyErrorResponse>,
    ),
> {
    state
        .token_service
        .revoke_token(token_id, &principal, &state.repositories.pool, &state.cache)
        .await
        .map_err(map_auth_error_to_json)?;

    Ok(StatusCode::NO_CONTENT)
}

fn map_auth_error_to_json(
    err: AuthError,
) -> (
    StatusCode,
    Json<crate::core::compatibility::error_response::FireflyErrorResponse>,
) {
    let (status, response) = map_auth_error(err);
    (status, Json(response))
}

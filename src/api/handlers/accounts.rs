use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AccountListRequest {
    pub page: Option<String>,
    pub limit: Option<String>,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
}

use crate::app::AppState;
use crate::core::auth::models::Principal;
use crate::core::compatibility::envelope::{FireflyListEnvelope, FireflySingleEnvelope};
use crate::core::compatibility::error_response::FireflyErrorResponse;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::{map_domain_error, DomainError};
use crate::core::persistence::repository::UserReadRepository;
use crate::modules::accounts::{
    AccountListQuery, CreateAccountRequest, FireflyAccountResource, UpdateAccountRequest,
};

async fn primary_currency_code(
    state: &Arc<AppState>,
    principal: &Principal,
) -> Result<String, DomainError> {
    state
        .repositories
        .user
        .find_by_id(&state.repositories.pool, principal.user_id)
        .await
        .map_err(|_| DomainError::Persistence)?
        .map(|user| user.primary_currency_code)
        .ok_or(DomainError::NotFound)
}

pub async fn list_accounts_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Query(request): Query<AccountListRequest>,
) -> Result<
    Json<FireflyListEnvelope<FireflyAccountResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let primary_currency_code = primary_currency_code(&state, &principal)
        .await
        .map_err(map_domain_error_to_json)?;
    let query = AccountListQuery::from_params(
        request.page.as_deref(),
        request.limit.as_deref(),
        request.account_type.as_deref(),
    )
    .map_err(map_domain_error_to_json)?;

    let result = state
        .account_service
        .list_accounts(query, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    let response = FireflyListEnvelope::from_paginated(Paginated {
        total_records: result.total_records,
        records: result
            .records
            .into_iter()
            .map(|record| FireflyAccountResource::from_view(record, &primary_currency_code))
            .collect(),
        current_page: result.current_page,
        per_page: result.per_page,
    });

    Ok(Json(response))
}

pub async fn get_account_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(account_id): Path<Uuid>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyAccountResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let primary_currency_code = primary_currency_code(&state, &principal)
        .await
        .map_err(map_domain_error_to_json)?;
    let result = state
        .account_service
        .get_account(account_id, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(Json(FireflySingleEnvelope {
        data: FireflyAccountResource::from_view(result, &primary_currency_code),
    }))
}

pub async fn create_account_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<
    (
        StatusCode,
        Json<FireflySingleEnvelope<FireflyAccountResource>>,
    ),
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let primary_currency_code = primary_currency_code(&state, &principal)
        .await
        .map_err(map_domain_error_to_json)?;
    let mut request = request;
    if request.currency_code.is_none() {
        request.currency_code = Some(primary_currency_code.clone());
    }

    let result = state
        .account_service
        .create_account(request, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok((
        StatusCode::CREATED,
        Json(FireflySingleEnvelope {
            data: FireflyAccountResource::from_view(result, &primary_currency_code),
        }),
    ))
}

pub async fn update_account_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(account_id): Path<Uuid>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyAccountResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let primary_currency_code = primary_currency_code(&state, &principal)
        .await
        .map_err(map_domain_error_to_json)?;
    let result = state
        .account_service
        .update_account(account_id, request, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(Json(FireflySingleEnvelope {
        data: FireflyAccountResource::from_view(result, &primary_currency_code),
    }))
}

pub async fn delete_account_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<FireflyErrorResponse>)> {
    state
        .account_service
        .delete_account(account_id, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(StatusCode::NO_CONTENT)
}

fn map_domain_error_to_json(err: DomainError) -> (StatusCode, Json<FireflyErrorResponse>) {
    let (status, response) = map_domain_error(err);
    (status, Json(response))
}

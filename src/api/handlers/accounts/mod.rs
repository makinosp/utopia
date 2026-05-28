use std::sync::Arc;

use axum::extract::{Path, Query, State};
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
use crate::modules::accounts::{
    AccountListQuery, AccountService, CreateAccountRequest, FireflyAccountResource,
    UpdateAccountRequest,
};

pub async fn list_accounts_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Query(request): Query<AccountListRequest>,
) -> Result<
    Json<FireflyListEnvelope<FireflyAccountResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let query = AccountListQuery::from_params(
        request.page.as_deref(),
        request.limit.as_deref(),
        request.account_type.as_deref(),
    )
    .map_err(map_domain_error_to_json)?;

    let service = AccountService::new(state.repositories.account.clone());
    let result = service
        .list_accounts(query, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    let response = FireflyListEnvelope::from_paginated(Paginated {
        total_records: result.total_records,
        records: result
            .records
            .into_iter()
            .map(FireflyAccountResource::from)
            .collect(),
        current_page: result.current_page,
        per_page: result.per_page,
    });

    Ok(Json(response))
}

pub async fn get_account_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Path(account_id): Path<Uuid>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyAccountResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let service = AccountService::new(state.repositories.account.clone());
    let result = service
        .get_account(account_id, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(Json(FireflySingleEnvelope {
        data: FireflyAccountResource::from(result),
    }))
}

pub async fn create_account_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<
    (
        StatusCode,
        Json<FireflySingleEnvelope<FireflyAccountResource>>,
    ),
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let service = AccountService::new(state.repositories.account.clone());
    let result = service
        .create_account(request, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok((
        StatusCode::CREATED,
        Json(FireflySingleEnvelope {
            data: FireflyAccountResource::from(result),
        }),
    ))
}

pub async fn update_account_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Path(account_id): Path<Uuid>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyAccountResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let service = AccountService::new(state.repositories.account.clone());
    let result = service
        .update_account(account_id, request, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(Json(FireflySingleEnvelope {
        data: FireflyAccountResource::from(result),
    }))
}

pub async fn delete_account_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<FireflyErrorResponse>)> {
    let service = AccountService::new(state.repositories.account.clone());
    service
        .delete_account(account_id, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(StatusCode::NO_CONTENT)
}

fn map_domain_error_to_json(err: DomainError) -> (StatusCode, Json<FireflyErrorResponse>) {
    let (status, response) = map_domain_error(err);
    (status, Json(response))
}

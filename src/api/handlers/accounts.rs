use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::app::AppState;
use crate::core::auth::models::Principal;
use crate::core::compatibility::envelope::FireflyListEnvelope;
use crate::core::compatibility::error_response::FireflyErrorResponse;
use crate::core::compatibility::pagination::Paginated;
use crate::core::error_mapping::mapper::{map_domain_error, DomainError};
use crate::modules::accounts::{AccountListQuery, AccountService, FireflyAccountResource};

#[derive(Debug, Deserialize)]
pub struct AccountListRequest {
    pub page: Option<String>,
    pub limit: Option<String>,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
}

pub async fn list_accounts_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(principal): axum::extract::Extension<Principal>,
    Query(request): Query<AccountListRequest>,
) -> Result<Json<FireflyListEnvelope<FireflyAccountResource>>, (StatusCode, Json<FireflyErrorResponse>)> {
    let query = AccountListQuery::from_params(
        request.page.as_deref(),
        request.limit.as_deref(),
        request.account_type.as_deref(),
    )
    .map_err(map_domain_error_to_json)?;

    let service = AccountService::new(state.repositories.account.clone());
    let accounts = service
        .list_accounts(query, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    let response = FireflyListEnvelope::from_paginated(Paginated {
        total_records: accounts.total_records,
        records: accounts
            .records
            .into_iter()
            .map(FireflyAccountResource::from)
            .collect(),
        current_page: accounts.current_page,
        per_page: accounts.per_page,
    });

    Ok(Json(response))
}

fn map_domain_error_to_json(
    err: DomainError,
) -> (StatusCode, Json<FireflyErrorResponse>) {
    let (status, response) = map_domain_error(err);
    (status, Json(response))
}

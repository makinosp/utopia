use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::core::auth::audit_logger::AuditLogger;
use crate::core::auth::models::Principal;
use crate::core::compatibility::envelope::{FireflyListEnvelope, FireflySingleEnvelope};
use crate::core::compatibility::error_response::FireflyErrorResponse;
use crate::core::compatibility::pagination::{Paginated, DEFAULT_LIMIT, DEFAULT_PAGE, MAX_LIMIT};
use crate::core::error_mapping::mapper::{map_domain_error, DomainError};
use crate::modules::transactions::{
    CreateTransactionRequest, FireflyTransactionResource, TransactionListQuery, TransactionService,
    UpdateTransactionRequest,
};

#[derive(Debug, Deserialize)]
pub struct TransactionListRequest {
    pub page: Option<String>,
    pub limit: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccountTransactionListRequest {
    pub page: Option<String>,
    pub limit: Option<String>,
}

pub async fn list_transactions_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Query(request): Query<TransactionListRequest>,
) -> Result<
    Json<FireflyListEnvelope<FireflyTransactionResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let query = TransactionListQuery::from_params(
        request.page.as_deref(),
        request.limit.as_deref(),
        request.start.as_deref(),
        request.end.as_deref(),
        request.transaction_type.as_deref(),
    )
    .map_err(map_domain_error_to_json)?;

    let service = TransactionService::new(
        state.repositories.transaction.clone(),
        state.repositories.account.clone(),
    );
    let result = service
        .list_transactions(query, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    let response = FireflyListEnvelope::from_paginated(Paginated {
        total_records: result.total_records,
        records: result
            .records
            .into_iter()
            .map(FireflyTransactionResource::from_view)
            .collect(),
        current_page: result.current_page,
        per_page: result.per_page,
    });

    Ok(Json(response))
}

pub async fn get_transaction_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(transaction_id): Path<Uuid>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyTransactionResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let service = TransactionService::new(
        state.repositories.transaction.clone(),
        state.repositories.account.clone(),
    );
    let result = service
        .get_transaction(transaction_id, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(Json(FireflySingleEnvelope {
        data: FireflyTransactionResource::from_view(result),
    }))
}

pub async fn create_transaction_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<
    (
        StatusCode,
        Json<FireflySingleEnvelope<FireflyTransactionResource>>,
    ),
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let service = TransactionService::new(
        state.repositories.transaction.clone(),
        state.repositories.account.clone(),
    );
    let result = service
        .create_transaction(request, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    state.audit_logger.emit(AuditLogger::new_event(
        "transaction_created",
        "success",
        Some(principal.user_id),
        None,
        None,
    ));

    Ok((
        StatusCode::CREATED,
        Json(FireflySingleEnvelope {
            data: FireflyTransactionResource::from_view(result),
        }),
    ))
}

pub async fn update_transaction_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(transaction_id): Path<Uuid>,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyTransactionResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let service = TransactionService::new(
        state.repositories.transaction.clone(),
        state.repositories.account.clone(),
    );
    let result = service
        .update_transaction(
            transaction_id,
            request,
            &principal,
            &state.repositories.pool,
        )
        .await
        .map_err(map_domain_error_to_json)?;

    state.audit_logger.emit(AuditLogger::new_event(
        "transaction_updated",
        "success",
        Some(principal.user_id),
        None,
        None,
    ));

    Ok(Json(FireflySingleEnvelope {
        data: FireflyTransactionResource::from_view(result),
    }))
}

pub async fn delete_transaction_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(transaction_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<FireflyErrorResponse>)> {
    let service = TransactionService::new(
        state.repositories.transaction.clone(),
        state.repositories.account.clone(),
    );
    service
        .delete_transaction(transaction_id, &principal, &state.repositories.pool)
        .await
        .map_err(map_domain_error_to_json)?;

    state.audit_logger.emit(AuditLogger::new_event(
        "transaction_deleted",
        "success",
        Some(principal.user_id),
        None,
        None,
    ));

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_account_transactions_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
    Path(account_id): Path<Uuid>,
    Query(request): Query<AccountTransactionListRequest>,
) -> Result<
    Json<FireflyListEnvelope<FireflyTransactionResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let page = parse_page_param(request.page)?;
    let limit = parse_limit_param(request.limit)?;

    let service = TransactionService::new(
        state.repositories.transaction.clone(),
        state.repositories.account.clone(),
    );
    let result = service
        .list_account_transactions(
            account_id,
            &principal,
            &state.repositories.pool,
            page,
            limit,
        )
        .await
        .map_err(map_domain_error_to_json)?;

    let response = FireflyListEnvelope::from_paginated(Paginated {
        total_records: result.total_records,
        records: result
            .records
            .into_iter()
            .map(FireflyTransactionResource::from_view)
            .collect(),
        current_page: result.current_page,
        per_page: result.per_page,
    });

    Ok(Json(response))
}

fn parse_page_param(raw: Option<String>) -> Result<u32, (StatusCode, Json<FireflyErrorResponse>)> {
    let raw = raw.as_deref();
    if raw.is_none() || raw.map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Ok(DEFAULT_PAGE);
    }
    let val = raw.unwrap().parse::<u32>().map_err(|_| {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "page".to_string(),
            vec!["The page field must be an integer.".to_string()],
        );
        let (status, response) = crate::core::error_mapping::mapper::map_validation_error(fields);
        (status, Json(response))
    })?;
    if val == 0 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "page".to_string(),
            vec!["The page field must be at least 1.".to_string()],
        );
        let (status, response) = crate::core::error_mapping::mapper::map_validation_error(fields);
        return Err((status, Json(response)));
    }
    Ok(val)
}

fn parse_limit_param(raw: Option<String>) -> Result<u32, (StatusCode, Json<FireflyErrorResponse>)> {
    let raw = raw.as_deref();
    if raw.is_none() || raw.map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Ok(DEFAULT_LIMIT);
    }
    let val = raw.unwrap().parse::<u32>().map_err(|_| {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "limit".to_string(),
            vec!["The limit field must be an integer.".to_string()],
        );
        let (status, response) = crate::core::error_mapping::mapper::map_validation_error(fields);
        (status, Json(response))
    })?;
    if val == 0 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "limit".to_string(),
            vec!["The limit field must be at least 1.".to_string()],
        );
        let (status, response) = crate::core::error_mapping::mapper::map_validation_error(fields);
        return Err((status, Json(response)));
    }
    if val > MAX_LIMIT {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "limit".to_string(),
            vec!["The limit field may not be greater than 100.".to_string()],
        );
        let (status, response) = crate::core::error_mapping::mapper::map_validation_error(fields);
        return Err((status, Json(response)));
    }
    Ok(val)
}

fn map_domain_error_to_json(err: DomainError) -> (StatusCode, Json<FireflyErrorResponse>) {
    let (status, response) = map_domain_error(err);
    (status, Json(response))
}

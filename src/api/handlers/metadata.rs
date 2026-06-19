use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::app::AppState;
use crate::core::auth::models::Principal;
use crate::core::compatibility::envelope::{FireflyListEnvelope, FireflySingleEnvelope};
use crate::core::compatibility::error_response::FireflyErrorResponse;
use crate::core::compatibility::pagination::{DEFAULT_LIMIT, DEFAULT_PAGE};
use crate::core::error_mapping::mapper::{map_domain_error, DomainError};
use crate::modules::metadata::{
    FireflyCurrencyResource, FireflySystemInfoResource, FireflyUserResource, MetadataService,
};

#[derive(Debug, Deserialize)]
pub struct CurrencyListRequest {
    pub page: Option<String>,
    pub limit: Option<String>,
}

pub async fn list_currencies_handler(
    _state: State<Arc<AppState>>,
    _principal: Extension<Principal>,
    Query(request): Query<CurrencyListRequest>,
) -> Result<
    Json<FireflyListEnvelope<FireflyCurrencyResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let page = parse_page_param(request.page, DEFAULT_PAGE)?;
    let limit = parse_limit_param(request.limit, DEFAULT_LIMIT)?;

    let response = MetadataService::list_currencies(page, limit);
    Ok(Json(response))
}

pub async fn get_about_user_handler(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<Principal>,
) -> Result<
    Json<FireflySingleEnvelope<FireflyUserResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let user = MetadataService::get_user(&state.repositories.pool, principal.user_id)
        .await
        .map_err(map_domain_error_to_json)?;

    Ok(Json(FireflySingleEnvelope { data: user }))
}

pub async fn get_about_handler(
    _state: State<Arc<AppState>>,
    _principal: Extension<Principal>,
) -> Result<
    Json<FireflyListEnvelope<FireflySystemInfoResource>>,
    (StatusCode, Json<FireflyErrorResponse>),
> {
    let response = MetadataService::get_system_info();
    Ok(Json(response))
}

fn parse_page_param(
    raw: Option<String>,
    default_value: u32,
) -> Result<u32, (StatusCode, Json<FireflyErrorResponse>)> {
    let raw = raw.as_deref();
    if raw.is_none() || raw.map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Ok(default_value);
    }
    let val = raw.unwrap().parse::<u32>().map_err(|_| {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "page".to_string(),
            vec!["The page field must be an integer.".to_string()],
        );
        let (status, response) = map_validation_error(fields);
        (status, Json(response))
    })?;
    if val == 0 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "page".to_string(),
            vec!["The page field must be at least 1.".to_string()],
        );
        let (status, response) = map_validation_error(fields);
        return Err((status, Json(response)));
    }
    Ok(val)
}

fn parse_limit_param(
    raw: Option<String>,
    default_value: u32,
) -> Result<u32, (StatusCode, Json<FireflyErrorResponse>)> {
    let raw = raw.as_deref();
    if raw.is_none() || raw.map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Ok(default_value);
    }
    let val = raw.unwrap().parse::<u32>().map_err(|_| {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "limit".to_string(),
            vec!["The limit field must be an integer.".to_string()],
        );
        let (status, response) = map_validation_error(fields);
        (status, Json(response))
    })?;
    if val == 0 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "limit".to_string(),
            vec!["The limit field must be at least 1.".to_string()],
        );
        let (status, response) = map_validation_error(fields);
        return Err((status, Json(response)));
    }
    if val > 100 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "limit".to_string(),
            vec!["The limit field may not be greater than 100.".to_string()],
        );
        let (status, response) = map_validation_error(fields);
        return Err((status, Json(response)));
    }
    Ok(val)
}

fn map_validation_error(
    fields: std::collections::HashMap<String, Vec<String>>,
) -> (StatusCode, FireflyErrorResponse) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        FireflyErrorResponse {
            message: "The given data was invalid.".to_string(),
            errors: fields,
        },
    )
}

fn map_domain_error_to_json(err: DomainError) -> (StatusCode, Json<FireflyErrorResponse>) {
    let (status, response) = map_domain_error(err);
    (status, Json(response))
}

use std::collections::HashMap;

use axum::http::StatusCode;

use crate::core::auth::error::AuthError;
use crate::core::compatibility::error_response::FireflyErrorResponse;

#[derive(Debug, Clone)]
pub enum DomainError {
    #[allow(dead_code)]
    NotFound,
    Validation(HashMap<String, Vec<String>>),
    Persistence,
    #[allow(dead_code)]
    Unexpected,
    #[allow(dead_code)]
    Conflict(String),
}

pub fn map_domain_error(err: DomainError) -> (StatusCode, FireflyErrorResponse) {
    match err {
        DomainError::Conflict(msg) => (
            StatusCode::CONFLICT,
            FireflyErrorResponse {
                message: msg,
                errors: HashMap::new(),
            },
        ),
        DomainError::NotFound => (
            StatusCode::NOT_FOUND,
            FireflyErrorResponse::new("Not Found"),
        ),
        DomainError::Validation(fields) => map_validation_error(fields),
        DomainError::Persistence | DomainError::Unexpected => (
            StatusCode::INTERNAL_SERVER_ERROR,
            FireflyErrorResponse::new("An unexpected error occurred."),
        ),
    }
}

pub fn map_auth_error(err: AuthError) -> (StatusCode, FireflyErrorResponse) {
    let reason = err.reason_code();
    let description = err.description();
    let message = format!("{reason}: {description}");
    (StatusCode::UNAUTHORIZED, FireflyErrorResponse::new(message))
}

pub fn map_validation_error(
    fields: HashMap<String, Vec<String>>,
) -> (StatusCode, FireflyErrorResponse) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        FireflyErrorResponse {
            message: "The given data was invalid.".to_string(),
            errors: fields,
        },
    )
}

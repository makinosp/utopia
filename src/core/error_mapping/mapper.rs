use std::collections::HashMap;

use axum::http::StatusCode;

use crate::core::auth::error::AuthError;
use crate::core::compatibility::error_response::FireflyErrorResponse;

#[derive(Debug, Clone)]
pub enum DomainError {
    NotFound,
    Validation(HashMap<String, Vec<String>>),
    Persistence,
    Unexpected,
}

pub fn map_domain_error(err: DomainError) -> (StatusCode, FireflyErrorResponse) {
    match err {
        DomainError::NotFound => (StatusCode::NOT_FOUND, FireflyErrorResponse::new("Not found.")),
        DomainError::Validation(fields) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            FireflyErrorResponse {
                message: "The given data was invalid.".to_string(),
                errors: fields,
            },
        ),
        DomainError::Persistence | DomainError::Unexpected => (
            StatusCode::INTERNAL_SERVER_ERROR,
            FireflyErrorResponse::new("An unexpected error occurred."),
        ),
    }
}

pub fn map_auth_error(err: AuthError) -> (StatusCode, FireflyErrorResponse) {
    let reason = err.reason_code();
    let message = format!("{}: {}", reason, err.description());
    (StatusCode::UNAUTHORIZED, FireflyErrorResponse::new(message))
}

pub fn map_validation_error(fields: HashMap<String, Vec<String>>) -> (StatusCode, FireflyErrorResponse) {
    map_domain_error(DomainError::Validation(fields))
}

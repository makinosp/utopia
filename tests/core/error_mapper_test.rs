use std::collections::HashMap;

use axum::http::StatusCode;
use utopia::core::auth::error::AuthError;
use utopia::core::error_mapping::mapper::{map_auth_error, map_domain_error, DomainError};

#[test]
fn maps_not_found_to_404() {
    let (status, body) = map_domain_error(DomainError::NotFound);
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.message, "Not found.");
}

#[test]
fn maps_validation_to_422_with_errors() {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), vec!["required".to_string()]);

    let (status, body) = map_domain_error(DomainError::Validation(fields));
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.errors.contains_key("name"));
}

#[test]
fn maps_auth_to_401_reason_code() {
    let (status, body) = map_auth_error(AuthError::TokenRevoked);
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.message.contains("token_revoked"));
}

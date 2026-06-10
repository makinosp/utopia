use axum::extract::Request;
use axum::http::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

/// Accept header validation middleware.
/// Ensures that the request Accept header, if present, is compatible with JSON APIs.
/// Firefly III supports: application/json, application/vnd.api+json
/// Default fallback: application/json
pub async fn accept_header_middleware(request: Request, next: Next) -> Response {
    validate_accept_header(request.headers());

    // Proceed to handler; response will be application/json per Axum default
    let mut response = next.run(request).await;

    // Ensure error and success responses include appropriate Content-Type
    if !response.headers().contains_key(CONTENT_TYPE) {
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
    }

    response
}

/// Validates the Accept header and logs warnings if it's malformed or unsupported.
fn validate_accept_header(headers: &HeaderMap) {
    // 1. Header is not present - nothing to validate
    let Some(accept_value) = headers.get(ACCEPT) else {
        return;
    };

    // 2. Header value cannot be parsed as UTF-8 string
    let Ok(accept_str) = accept_value.to_str() else {
        warn!("Malformed Accept header; proceeding with default");
        return;
    };

    // 3. Check if Accept header contains at least one compatible media type
    let has_json = accept_str.split(',').any(|part| {
        let trimmed = part.trim();
        trimmed.starts_with("application/json")
            || trimmed.starts_with("application/vnd.api+json")
            || trimmed == "*/*"
    });

    if !has_json {
        warn!(
            accept = accept_str,
            "Unsupported Accept header; will proceed with JSON response"
        );
    }
}

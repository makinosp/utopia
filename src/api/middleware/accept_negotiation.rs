use axum::extract::Request;
use axum::http::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

/// Accept header validation middleware.
/// Ensures that the request Accept header, if present, is compatible with JSON APIs.
/// Firefly III supports: application/json, application/vnd.api+json
/// Default fallback: application/json
pub async fn accept_header_middleware(request: Request, next: Next) -> Response {
    let accept = request.headers().get(ACCEPT);

    if let Some(accept_value) = accept {
        match accept_value.to_str() {
            Ok(accept_str) => {
                // Validate that the Accept header contains at least one compatible media type
                let parts: Vec<&str> = accept_str.split(',').collect();
                let has_json = parts.iter().any(|part| {
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
                    // Firefly behavior: fallback to JSON rather than returning 406
                }
            }
            Err(_) => {
                warn!("Malformed Accept header; proceeding with default");
            }
        }
    }

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

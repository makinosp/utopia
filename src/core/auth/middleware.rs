use std::sync::Arc;

use axum::extract::State;
use axum::http::{header::AUTHORIZATION, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;

use crate::app::AppState;
use crate::core::auth::audit_logger::AuditLogger;
use crate::core::auth::validator::validate_bearer;
use crate::core::error_mapping::mapper::map_auth_error;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let authorization = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match validate_bearer(
        authorization,
        &state.repositories.token,
        &state.repositories.user,
        &state.repositories.pool,
        &state.cache,
        &state.metrics,
    )
    .await
    {
        Ok(principal) => {
            let event = AuditLogger::new_event(
                "auth_validation",
                "success",
                Some(principal.user_id),
                None,
                request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string()),
            );
            state.audit_logger.emit(event);
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(err) => {
            state
                .metrics
                .auth_failures_total
                .with_label_values(&[err.reason_code()])
                .inc();

            let event = AuditLogger::new_event(
                "auth_validation",
                "failure",
                None,
                Some(err.reason_code()),
                request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string()),
            );
            state.audit_logger.emit(event);

            let (status, body) = map_auth_error(err);
            (status, axum::Json(body)).into_response()
        }
    }
}

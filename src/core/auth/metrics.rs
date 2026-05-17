use axum::http::{header::CONTENT_TYPE, StatusCode};
use axum::response::{IntoResponse, Response};
use prometheus::{Encoder, Histogram, HistogramOpts, IntCounter, IntCounterVec, Opts, Registry, TextEncoder};

#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    registry: Registry,
    pub auth_validation_latency_ms: Histogram,
    pub authenticated_requests_total: IntCounter,
    pub auth_failures_total: IntCounterVec,
    pub auth_cache_hit_total: IntCounterVec,
    pub auth_cache_miss_total: IntCounter,
    pub auth_dependency_failure_total: IntCounter,
    pub token_issue_total: IntCounter,
    pub token_revoke_total: IntCounter,
    pub http_5xx_total: IntCounter,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let auth_validation_latency_ms = Histogram::with_opts(HistogramOpts::new(
            "auth_validation_latency_ms",
            "Auth validation latency in milliseconds",
        ))
        .expect("histogram");

        let authenticated_requests_total = IntCounter::with_opts(Opts::new(
            "authenticated_requests_total",
            "Total authenticated requests",
        ))
        .expect("counter");

        let auth_failures_total = IntCounterVec::new(
            Opts::new("auth_failures_total", "Auth failures by reason code"),
            &["reason_code"],
        )
        .expect("counter vec");

        let auth_cache_hit_total = IntCounterVec::new(
            Opts::new("auth_cache_hit_total", "Auth cache hits by cache type"),
            &["cache_type"],
        )
        .expect("counter vec");

        let auth_cache_miss_total = IntCounter::with_opts(Opts::new(
            "auth_cache_miss_total",
            "Auth cache miss count",
        ))
        .expect("counter");

        let auth_dependency_failure_total = IntCounter::with_opts(Opts::new(
            "auth_dependency_failure_total",
            "Dependency failures in auth path",
        ))
        .expect("counter");

        let token_issue_total = IntCounter::with_opts(Opts::new("token_issue_total", "Issued token count"))
            .expect("counter");

        let token_revoke_total = IntCounter::with_opts(Opts::new("token_revoke_total", "Revoked token count"))
            .expect("counter");

        let http_5xx_total = IntCounter::with_opts(Opts::new("http_5xx_total", "HTTP 5xx count"))
            .expect("counter");

        for collector in [
            Box::new(auth_validation_latency_ms.clone()) as Box<dyn prometheus::core::Collector>,
            Box::new(authenticated_requests_total.clone()),
            Box::new(auth_failures_total.clone()),
            Box::new(auth_cache_hit_total.clone()),
            Box::new(auth_cache_miss_total.clone()),
            Box::new(auth_dependency_failure_total.clone()),
            Box::new(token_issue_total.clone()),
            Box::new(token_revoke_total.clone()),
            Box::new(http_5xx_total.clone()),
        ] {
            registry.register(collector).expect("register collector");
        }

        Self {
            registry,
            auth_validation_latency_ms,
            authenticated_requests_total,
            auth_failures_total,
            auth_cache_hit_total,
            auth_cache_miss_total,
            auth_dependency_failure_total,
            token_issue_total,
            token_revoke_total,
            http_5xx_total,
        }
    }

    pub fn render(&self) -> Result<String, prometheus::Error> {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

pub async fn metrics_handler(
    state: axum::extract::State<std::sync::Arc<crate::app::AppState>>,
) -> Response {
    match state.metrics.render() {
        Ok(body) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
            body,
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "failed to render prometheus metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

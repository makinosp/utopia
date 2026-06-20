# Auth Enhancement — NFR Design Plan (US-021 / US-022)

## Background
This plan covers the NFR design decisions for the auth enhancement unit, translating NFR requirements into concrete design patterns and logical component layouts for rate limiting, integration test architecture, and PBT integration.

**Input Artifacts**:
- NFR Requirements: `aidlc-docs/construction/auth-enhancement/nfr-requirements/nfr-requirements.md`
- Tech Stack Decisions: `aidlc-docs/construction/auth-enhancement/nfr-requirements/tech-stack-decisions.md`
- Existing Code: `src/core/auth/`, `src/api/`, `src/config.rs`

## Design Checklist

- [x] Verify NFR requirements coverage in design patterns
- [x] Define rate limiter logical component placement
- [x] Specify rate limiter data structure and concurrency model
- [x] Define HTTP 429 error response integration path
- [x] Design audit logging integration for rate limit events
- [x] Design Prometheus metrics integration for rate limit counter
- [x] Define integration test architecture for auth flows
- [x] Define PBT test structure for token format and error serialization
- [x] Validate Security Baseline extension compliance
- [x] Validate Property-Based Testing extension compliance

## NFR Design Questions

### Question NFRD-1: Rate Limiter Component Placement
Where should the rate limiting logic be placed in the existing component structure?

A) As a new Axum middleware layer (`src/api/middleware/rate_limiter.rs`) that wraps the bootstrap token route — applied via the router at `src/api/router.rs`

B) As a service-layer guard inside `src/core/auth/service.rs`, invoked directly before token generation logic

C) As a separate `RateLimiter` component in `src/core/auth/rate_limiter.rs` with its own struct, injected into the bootstrap token handler

D) As an inline check inside the existing bootstrap token handler (`src/api/handlers/tokens.rs`), with no separate abstraction

[Answer]: A

### Question NFRD-2: Rate Limiter Concurrency Model
How should the rate limiter's shared state be managed for thread safety?

A) `Arc<RwLock<HashMap<IpAddr, RateLimitState>>>` — shared state passed via Axum application state

B) `Arc<Mutex<HashMap<IpAddr, RateLimitState>>>` — exclusive lock for simplicity at low concurrency

C) Per-worker sharded counters with `dashmap::DashMap` — lock-free concurrent map (adds `dashmap` dependency)

D) `tokio::sync::RwLock<HashMap<IpAddr, RateLimitState>>` — async-aware lock for cooperative scheduling

[Answer]: A

### Question NFRD-3: Stale Entry Eviction Strategy
How should stale rate limit entries be evicted to prevent unbounded memory growth?

A) Periodic background task (spawned via `tokio::spawn`) that runs every 60 seconds and removes entries older than 2x the window size

B) Lazy eviction on each check — if an entry's window has expired, clear and restart the window

C) Both A and B — eager periodic cleanup + lazy eviction on access

D) Fixed-size LRU cache using `lru` crate — evicts oldest entries when capacity is reached (adds dependency)

[Answer]: C

### Question NFRD-4: Error Response Integration
How should the HTTP 429 response be integrated with the existing error handling?

A) Extend the existing `auth.rs` error enum with a `RateLimitExceeded` variant, handle via the existing error-to-response mapper (`src/core/error_mapping/`)

B) Return HTTP 429 directly from the handler without going through the error mapping layer — a fast-path response

C) Create a dedicated `RateLimitError` type in the rate limiter module, convert to Axum response via `IntoResponse` impl

D) Use the existing middleware error handler, mapping the rate limit rejection to a standard error response

[Answer]: A

### Question NFRD-5: Audit Logging Integration
How should rate limit hits be integrated with the existing structured audit logging?

A) Emit a `tracing::warn!` event with structured fields (source_ip, endpoint, reason_code, window_requests, window_limit) — rely on the existing JSON log subscriber

B) Add a dedicated `audit_rate_limit` method to the existing audit logger component, called from the rate limiter

C) Use `tracing::info!` for rate limit events to distinguish from error-level auth failures

D) Log via the existing security event channel (the same mechanism used for auth failures per SEC-003)

[Answer]: D

### Question NFRD-6: Prometheus Metrics Integration
How should the rate limit counter be added to the existing Prometheus metrics setup?

A) Register `utopia_rate_limited_requests_total` as a new counter in the existing metrics module, increment from the rate limiter on each rejection

B) Add a `RateLimitMetrics` struct wrapping the counter, register in `src/core/auth/metrics.rs`

C) Reuse the existing `auth_failures_total` counter with a new `reason_code=rate_limit_exceeded` label — no new metric name

D) Both A and B — new counter in existing module, wrapped in a dedicated struct for clarity

[Answer]: A

### Question NFRD-7: Integration Test Architecture
How should the auth integration tests be structured?

A) Single `tests/auth_integration_test.rs` covering all three test scopes (bootstrap cycle, 401 checks, rate limit enforcement)

B) Separate test files: `tests/auth_bootstrap_test.rs`, `tests/auth_401_test.rs`, `tests/auth_rate_limit_test.rs`

C) Add auth-specific test helpers to the existing `tests/core/support.rs` and write tests in `tests/auth_integration_test.rs` using shared infrastructure

D) Write integration tests as a separate `auth-test` crate to keep auth tests isolated

[Answer]: C

### Question NFRD-8: Environment Variable Naming Convention
What naming convention should the rate limit env vars follow?

A) `BOOTSTRAP_RATE_LIMIT_REQUESTS` / `BOOTSTRAP_RATE_LIMIT_WINDOW_SECS` — prefixed with `BOOTSTRAP_` to scope to the bootstrap endpoint

B) `RATE_LIMIT_BOOTSTRAP_REQUESTS` / `RATE_LIMIT_BOOTSTRAP_WINDOW_SECS` — prefixed with `RATE_LIMIT_` for grouping, suffixed for scope

C) `APP_RATE_LIMIT_BOOTSTRAP_REQUESTS` / `APP_RATE_LIMIT_BOOTSTRAP_WINDOW_SECS` — use the existing `APP_` prefix pattern (consistent with `APP_STRICT_SSL`)

D) `UTOPIA_RATE_LIMIT_BOOTSTRAP_REQUESTS` / `UTOPIA_RATE_LIMIT_BOOTSTRAP_WINDOW_SECS` — use the project name as prefix

[Answer]: C

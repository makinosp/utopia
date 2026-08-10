# Auth Enhancement — Code Generation Plan (UOW-04 / US-021, US-022)

## Unit Context
- **Unit**: UOW-04 Auth Enhancement
- **Stories**: US-021 (Rate Limiting on Bootstrap Endpoint), US-022 (Integration Tests & Observability)
- **Dependencies**: UOW-01 Core Foundation (existing auth, router, config infrastructure)
- **Design Artifacts**:
  - NFR Requirements: `aidlc-docs/construction/auth-enhancement/nfr-requirements/nfr-requirements.md`
  - Tech Stack Decisions: `aidlc-docs/construction/auth-enhancement/nfr-requirements/tech-stack-decisions.md`
  - NFR Design Patterns: `aidlc-docs/construction/auth-enhancement/nfr-design/nfr-design-patterns.md`
  - Logical Components: `aidlc-docs/construction/auth-enhancement/nfr-design/logical-components.md`

## Files to Modify
| File | Action | Description |
|------|--------|-------------|
| `src/core/auth/error.rs` | MODIFY | Add `RateLimitExceeded` variant and update `reason_code()`/`description()` |
| `src/core/error_mapping/mapper.rs` | MODIFY | Handle `RateLimitExceeded` → HTTP 429 with Retry-After header and Firefly-III compatible body |
| `src/core/auth/metrics.rs` | MODIFY | Add `utopia_rate_limited_requests_total` counter |
| `src/config.rs` | MODIFY | Add `bootstrap_rate_limit_requests` and `bootstrap_rate_limit_window_secs` fields |
| `src/app.rs` | MODIFY | Inject `RateLimitState` into AppState |
| `src/api/router.rs` | MODIFY | Apply rate limit middleware to the bootstrap token route |
| `.env.example` | MODIFY | Document new env vars |

## Files to Create
| File | Action | Description |
|------|--------|-------------|
| `src/api/middleware/rate_limiter.rs` | CREATE | Rate limit middleware module |
| `tests/auth_integration_test.rs` | CREATE | Integration tests for auth flows (bootstrap cycle, 401, rate limit) |

## Files to Extend (PBT)
| File | Action | Description |
|------|--------|-------------|
| `tests/core/error_mapper_test.rs` | MODIFY | Add PBT for `AuthError` serialization round-trip (TST-002) |
| `tests/core/token_lifecycle_test.rs` | MODIFY | Add PBT for token format validation round-trip (TST-002) |

---

## Step-by-Step Execution Plan

### Step 1: Add `RateLimitExceeded` variant to `AuthError`
- [x] Add `RateLimitExceeded` variant to `AuthError` enum
- [x] Add `RateLimitExceeded { retry_after_secs: u64 }` with reason_code `"rate_limit_exceeded"` and description
- [x] Update `reason_code()` and `description()` match arms

### Step 2: Create rate limiter module (`src/api/middleware/rate_limiter.rs`)
- [x] Define `RateLimitState` struct (wraps `Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>`)
- [x] Define `RateLimitEntry` struct (`window_start: Instant, count: u64`)
- [x] Implement `RateLimitState::new(requests: u64, window_secs: u64) -> Self`
- [x] Implement `RateLimitState::check_and_count(&self, ip: IpAddr) -> Result<(), AuthError>` — fixed-window logic with lazy eviction
- [x] Implement `RateLimitState::run_eviction_task(self: Arc<Self>)` — background periodic cleanup every 60s
- [x] Implement the Axum middleware function `rate_limit_middleware` — extracts IP from headers, calls `check_and_count`, on rejection: logs audit event, increments metrics, returns HTTP 429

### Step 3: Update error mapper (`src/core/error_mapping/mapper.rs`)
- [x] Modify `map_auth_error` to handle `AuthError::RateLimitExceeded` — return HTTP 429 with Retry-After header
- [x] Return Firefly-III compatible body: `{ "message": "...", "exception": "RateLimitExceededException" }`
- [x] Note: Retry-After header is added at the middleware level, not the mapper

### Step 4: Update metrics (`src/core/auth/metrics.rs`)
- [x] Add `pub rate_limited_requests_total: IntCounterVec` field
- [x] Initialize in `new()` with name `"utopia_rate_limited_requests_total"` and labels `["endpoint", "reason"]`
- [x] Register in registry
- [x] Add to `PrometheusMetrics` struct

### Step 5: Update config (`src/config.rs`)
- [x] Add fields: `bootstrap_rate_limit_requests: u64` (default 5), `bootstrap_rate_limit_window_secs: u64` (default 60)
- [x] Parse from env vars `APP_RATE_LIMIT_BOOTSTRAP_REQUESTS` and `APP_RATE_LIMIT_BOOTSTRAP_WINDOW_SECS`

### Step 6: Update app state (`src/app.rs`)
- [x] Add `rate_limit_state: Arc<RateLimitState>` to `AppState`
- [x] Initialize in `build_app()` using config values
- [x] Spawn the eviction background task

### Step 7: Update router integration (`src/api/router.rs`)
- [x] Import rate limit middleware
- [x] Apply `rate_limit_middleware` to the bootstrap token route (route-layer middleware)

### Step 8: Update `.env.example`
- [x] Add `APP_RATE_LIMIT_BOOTSTRAP_REQUESTS=5`
- [x] Add `APP_RATE_LIMIT_BOOTSTRAP_WINDOW_SECS=60`

### Step 9: Create integration tests (`tests/auth_integration_test.rs`)
- [x] Add test for full bootstrap → token issuance cycle via HTTP
- [x] Add test for HTTP 401 on protected endpoints without token
- [x] Add test for rate limit enforcement — exceed window and verify HTTP 429 + Retry-After header + error body
- [x] Add test helpers to `tests/core/support.rs` (if needed)

### Step 10: Extend PBT for error serialization (`tests/core/error_mapper_test.rs`)
- [x] Add `proptest` strategy for generating all `AuthError` variants (including `RateLimitExceeded`)
- [x] Verify round-trip: `AuthError` → JSON response → parsed fields contain expected reason_code

### Step 11: Extend PBT for token format validation (`tests/core/token_lifecycle_test.rs`)
- [x] Add `proptest` test that any generated token string round-trips through format validation
- [x] Strategy: generate random byte sequences, encode to URL-safe base64, decode back

### Step 12: Verify compilation
- [x] Run `cargo check` to verify no compile errors
- [x] Fix any compilation issues

## Extension Compliance
- **Security Baseline**: Rate limit mechanism follows fail-open on counter overflow, no persistence of state across restarts, structured audit logging on rate limit hits per SEC-003 format
- **Property-Based Testing (Partial)**: PBT coverage for token format round-trip and auth error serialization as specified in TST-002

## Story Traceability
| Story | Step(s) | Description |
|-------|---------|-------------|
| US-021 | Steps 1–8 | Rate limiting on bootstrap token endpoint |
| US-022 | Steps 9–11 | Integration tests, PBT, observability |
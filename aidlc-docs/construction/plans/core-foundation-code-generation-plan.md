# Code Generation Plan - Core Foundation (UOW-01)

## Unit Context

| Field | Value |
|---|---|
| Unit ID | UOW-01 |
| Unit Name | Core Foundation |
| Delivery Order | 1 of 5 |
| Primary Stories | US-021 (Token Issuance), US-022 (Reject Unauthenticated Requests) |
| Secondary Coverage | Cross-cutting contracts required by UOW-02 through UOW-05 |
| Language | Rust |
| API Framework | Axum |
| Database | PostgreSQL via SQLx |
| Application Code Root | <REPO_ROOT> |

## Unit Dependencies
- None (this is the first delivery unit)
- UOW-02 through UOW-05 will depend on interfaces and traits generated here

## Story Traceability

| Story | Title | Covered By |
|---|---|---|
| US-021 | Obtain OAuth2/Personal Access Token | Steps 12, 13 (token issuance service + handler) |
| US-022 | Reject Unauthenticated Requests | Steps 10, 11 (auth validator + auth middleware) |

## Code Layout
Following the approved greenfield single-package layout:

```text
<REPO_ROOT>/
  Cargo.toml
  Cargo.lock
  .env.example
  docker/
    docker-compose.yml
    caddy/Caddyfile
    prometheus/prometheus.yml
    grafana/
    loki/loki-config.yml
    promtail/promtail-config.yml
  migrations/
    0001_initial_schema.sql
  src/
    main.rs
    app.rs
    config.rs
    core/
      mod.rs
      auth/
        mod.rs
        middleware.rs
        validator.rs
        cache.rs
        models.rs
        error.rs
      compatibility/
        mod.rs
        pagination.rs
        envelope.rs
        error_response.rs
      error_mapping/
        mod.rs
        mapper.rs
      persistence/
        mod.rs
        db.rs
        transaction_manager.rs
        repository.rs
    modules/  (empty placeholder mods for UOW-02..05)
      mod.rs
    api/
      mod.rs
      router.rs
      handlers/
        mod.rs
        tokens.rs
  tests/
    core/
      auth_validator_test.rs
      error_mapper_test.rs
      pagination_test.rs
      token_lifecycle_test.rs
      decimal_serialization_test.rs  (PBT scope)
  aidlc-docs/construction/core-foundation/code/
    code-summary.md
```

> Note: `<REPO_ROOT>` denotes the repository root in documentation. Do not commit local absolute filesystem paths.

## Generation Steps

### Step 1 — Project Scaffold
- [ ] Create `Cargo.toml` with workspace-ready single-crate setup
- [ ] Pin all dependency versions (no `latest`; use exact or range-locked versions)
- [ ] Include: axum, tokio, serde, serde_json, sqlx (postgres, runtime-tokio), argon2,
  uuid, rust_decimal, tracing, tracing-subscriber, prometheus, tower, tower-http,
  anyhow, thiserror, dotenvy, rand, moka (with time and tokio features), sha2, proptest
- [ ] Verify the chosen `moka` release supports the intended async/time feature names; adjust the feature set to the version-specific names before generating `Cargo.toml`
- [ ] Create `Cargo.lock` (generated on first build)
- [ ] Create `.env.example` documenting all required environment variables, including bootstrap-specific settings
- [ ] Create `src/main.rs` — entry point loading config and launching the Axum server
- Story coverage: foundational prerequisite

### Step 2 — Config Validator (`src/config.rs`)
- [ ] Define `AppConfig` struct loaded from environment variables
- [ ] Include: `DATABASE_URL`, `ARGON2_MEMORY_COST`, `ARGON2_TIME_COST`, `ARGON2_PARALLELISM`,
  `TOKEN_CACHE_TTL_SECS`, `APP_PORT`, `LOG_LEVEL`, `BOOTSTRAP_KEY`, `BOOTSTRAP_USER_EMAIL`
- [ ] Validate Argon2id parameters against minimum safe bounds at startup
  (memory >= 65536, time >= 3, parallelism >= 1)
- [ ] Return clear startup error with parameter name if validation fails
- Story coverage: SEC-001, SEC-002 enforcement at startup

### Step 3 — Database Connection (`src/core/persistence/db.rs`)
- [ ] Create `PgPool` initializer using `sqlx::postgres::PgPoolOptions`
- [ ] Enforce TLS mode for database connections via `ssl_mode=require` in connection string
- [ ] Expose `create_pool(config: &AppConfig) -> Result<PgPool, sqlx::Error>`
- [ ] Apply `migrations/0001_initial_schema.sql` automatically during startup using `sqlx::migrate!()` after pool creation; document this as the default migration path
- Story coverage: SECURITY-01, REL-003 infrastructure

### Step 4 — Database Migration (`migrations/0001_initial_schema.sql`)
- [ ] Create `users` table:
  - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
  - `email TEXT NOT NULL UNIQUE`
  - `blocked BOOLEAN NOT NULL DEFAULT FALSE`
  - `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- [ ] Create `personal_access_tokens` table:
  - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
  - `user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE`
  - `label TEXT NOT NULL`
  - `token_sha256 TEXT NOT NULL UNIQUE`
  - `token_hash TEXT NOT NULL`
  - `status TEXT NOT NULL DEFAULT 'Active'` (values: Active, Revoked)
  - `last_used_at TIMESTAMPTZ`
  - `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- [ ] Create index on `personal_access_tokens(token_sha256)` for O(1) lookup
- Story coverage: US-021 data model, US-022 lookup path

### Step 5 — Repository Interface (`src/core/persistence/repository.rs`)
- [ ] Define `TokenRepository` trait with read and write interface seam:
  ```rust
  pub trait TokenReadRepository { ... }
  pub trait TokenWriteRepository { ... }
  ```
  Methods: `find_by_sha256`, `find_by_id`, `create_token`, `revoke_token`
- [ ] Define `UserReadRepository` trait:
  Methods: `find_by_id`, `find_by_email`
- [ ] Define `UserWriteRepository` trait for bootstrap provisioning:
  Methods: `create_user`
- [ ] Define `TokenUpdateRepository` trait for fire-and-forget updates:
  Methods: `update_last_used_at(&self, token_sha256: &str) -> Result<(), sqlx::Error>`
- [ ] Implement `PgTokenRepository` and `PgUserRepository` using `sqlx`; the user repository must support both lookup and bootstrap user creation paths
- [ ] All queries use parameterized statements (no string concatenation)
- Story coverage: BR-AUTH-002 token lookup, BR-AUTHZ-001 ownership filter, SECURITY-05

### Step 6 — Transaction Manager (`src/core/persistence/transaction_manager.rs`)
- [ ] Implement `TransactionManager` backed by `sqlx::Transaction<'_, Postgres>`
- [ ] Expose `in_request_tx<T, F>(pool, work) -> Result<T, DomainError>`
- [ ] Transaction commits on success, rolls back on any error path
- Story coverage: BR-TX-001, BR-TX-002, BR-TX-003, REL-002

### Step 7 — Core Domain Types and Error Taxonomy (`src/core/auth/models.rs`, `src/core/auth/error.rs`)
- [ ] Define `Principal { user_id: Uuid, email: String }`
- [ ] Define `TokenRecord`, `UserRecord` value objects
- [ ] Define `AuthError` enum with reason codes:
  `MissingAuthorizationHeader`, `TokenMalformed`, `TokenNotFound`, `TokenRevoked`, `UserBlocked`
- [ ] Derive standard traits; keep types free of I/O dependencies
- Story coverage: BR-AUTH-001 through BR-AUTH-004, BR-ERR-004

### Step 8 — Compatibility Primitives (`src/core/compatibility/`)
- [ ] `pagination.rs`: `PaginationMeta`, `Paginated<T>`, `compute_pagination()` helper
- [ ] `envelope.rs`: `FireflyListEnvelope<T>`, `FireflySingleEnvelope<T>` wrappers
- [ ] `error_response.rs`: `FireflyErrorResponse { message: String, errors: HashMap<String, Vec<String>> }`
- [ ] `DecimalAmount` newtype wrapping `rust_decimal::Decimal` with serde using string round-trip
- Story coverage: BR-PAG-001 through BR-PAG-003, BR-ERR-001 through BR-ERR-003, BR-DTO-003

### Step 9 — Error Mapper (`src/core/error_mapping/mapper.rs`)
- [ ] Implement `map_domain_error(DomainError) -> (StatusCode, FireflyErrorResponse)`
- [ ] Implement `map_auth_error(AuthError) -> (StatusCode, FireflyErrorResponse)`
- [ ] Implement `map_validation_error(fields: HashMap<..>) -> (StatusCode, FireflyErrorResponse)`
- [ ] Follow BR-ERR-002 HTTP status assignment table exactly
- Story coverage: Flow 4 (Error Mapping), BR-ERR-001 through BR-ERR-004

### Step 10 — Auth Validator (`src/core/auth/validator.rs`)
- [ ] Implement `validate_bearer(raw_token, repo, cache, pool) -> Result<Principal, AuthError>`:
  1. Extract token from `Bearer <value>` format
  2. Reject malformed format with `TokenMalformed`
  3. Compute a fast `sha256_token` fingerprint from the raw token and check cache (both positive and negative entries)
  4. On cache hit: extract `user_id` from the cache entry; resolve the full `Principal { user_id, email }` via `UserReadRepository::find_by_id`; return the cached result (or cached error for negative cache) without Argon2 work
  5. On cache miss: query repository by `token_sha256`
  6. If a record exists and `status = Revoked`, return `TokenRevoked` immediately when the `token_sha256` matches; only non-revoked records proceed to Argon2id verification
  7. If verification succeeds, load owning user; reject if `blocked = true`
  8. Cache valid result (positive) or invalid result (negative with short TTL)
  9. Spawn fire-and-forget task for `last_used_at` update using `PgPool` clone (not transaction)
  10. If the background update fails, record `auth_dependency_failure_total` and emit `tracing::error!` with safe identifiers only; never log raw tokens
  11. Return `Principal`
- [ ] Fail closed on persistence errors (no in-request retries)
- [ ] Emit `auth_dependency_failure_total` metric on persistence failure
- [ ] Spawn task captures `Arc<PrometheusMetrics>` for error recording in background task
- Story coverage: US-022, BR-AUTH-001, BR-AUTH-002, REL-003, Pattern R-01, R-02

### Step 11 — Token Cache (`src/core/auth/cache.rs`)
- [ ] Implement `TokenCacheEntry` struct containing `user_id`, `token_status`, `cached_at`; keep `email` out of cache entries to reduce long-lived PII retention
- [ ] Implement `CachedAuthResult` enum: `Valid(TokenCacheEntry)` | `Invalid { reason: AuthError }`
- [ ] Use `sha256_token` as key (not Argon2 hash) to avoid expensive Argon2 computation on cache hit
- [ ] Implement positive cache: valid tokens with TTL from `TOKEN_CACHE_TTL_SECS`
- [ ] Implement negative cache: invalid/malformed/revoked tokens with short TTL (e.g., 60s)
- [ ] Configure max cache size (e.g., 10,000 entries) to prevent unbounded growth
- [ ] Expose `get(&sha256_token) -> Option<CachedAuthResult>` async method
- [ ] Expose `insert_valid(sha256_token, entry)` and `insert_invalid(sha256_token, error)` methods
- [ ] Expose `invalidate(sha256_token)` API called on revocation
- [ ] Expose `invalidate_all()` only for an explicit global sign-out or mass-revocation operation; if no such product requirement is approved, omit the API and rely on per-token invalidation
- Story coverage: Pattern P-01, P-02, PRF-001

### Step 12 — Auth Middleware Facade (`src/core/auth/middleware.rs`)
- [ ] Implement Axum middleware using `tower::Layer` / `axum::middleware::from_fn`
- [ ] Extract `Authorization` header; invoke `Auth Validator`
- [ ] On success: insert `Principal` into request extensions
- [ ] On failure: call error mapper and return short-circuit response
- [ ] Emit metrics on both success and failure outcomes
- [ ] Pass all outcomes through Audit Logger
- Story coverage: US-022 end-to-end authentication guard

### Step 13 — Token Issuance Service and Handler (`src/core/auth/`, `src/api/handlers/tokens.rs`)
- [ ] Define `TokenIssuanceResponse` JSON schema:
  ```json
  {
    "data": {
      "id": "uuid",
      "label": "string",
      "token": "string (raw token, shown only once)",
      "status": "Active",
      "created_at": "ISO8601 timestamp"
    }
  }
  ```
- [ ] Implement `TokenService::issue_token(label, principal, repo, tx_manager)`:
  1. Generate 32+ bytes of cryptographic randomness (`rand::thread_rng`)
  2. Encode as URL-safe base64
  3. Compute `sha256_token` from the raw token and persist it for fast validator lookup
  4. Hash with Argon2id using validated parameters from config
  5. Persist `TokenRecord` (status: Active) inside transaction boundary, including both `token_sha256` and `token_hash`
  6. Return `TokenIssuanceResponse` with raw token value + record metadata
- [ ] Document idempotency: token issuance is not idempotent; repeated requests produce new tokens
- [ ] Implement `TokenService::revoke_token(token_id, principal, repo, tx_manager, cache)`:
  1. Look up token by ID
  2. Verify ownership (caller is token owner OR has Owner role)
  3. Set status to `Revoked`; persist update
  4. Invalidate token cache entry by `token_sha256`
  5. Return HTTP 204
- [ ] Implement bootstrap issuance path for first-access token:
  - Accept `X-Bootstrap-Key` and compare it to the configured bootstrap secret using constant-time equality; do not log or echo the raw value
  - Create or locate the bootstrap identity/user via `UserReadRepository::find_by_email` and `UserWriteRepository::create_user` before issuing the first PAT
  - Use `BOOTSTRAP_USER_EMAIL` from configuration as the bootstrap user identifier for lookup/creation
  - Return token payload once and do not store raw token
  - Restrict bootstrap endpoint to a one-time or rolling bootstrap key
- [ ] Wire `POST /api/v1/tokens` (issuance) and `DELETE /api/v1/tokens/{id}` (revocation) in `src/api/handlers/tokens.rs`
- [ ] Wire `POST /api/v1/bootstrap/tokens` as bootstrap token issuance endpoint in `src/api/handlers/tokens.rs`
- Story coverage: US-021, BR-AUTH-003, BR-AUTH-004, BR-AUTHZ-002, SEC-001, SEC-002

### Step 14 — Metrics and Audit Logger (`src/core/auth/`)
- [ ] Register Prometheus counters and histograms matching OBS-002 metric names:
  - `auth_validation_latency_ms` (histogram)
  - `authenticated_requests_total` (counter)
  - `auth_failures_total` labeled by `reason_code`
  - `auth_cache_hit_total` labeled by `cache_type` (positive/negative)
  - `auth_cache_miss_total`
  - `auth_dependency_failure_total`
  - `token_issue_total`, `token_revoke_total`
  - `http_5xx_total`
- [ ] Create `PrometheusMetrics` struct wrapping all counters/histograms
- [ ] Wrap `PrometheusMetrics` in `Arc` for thread-safe sharing with spawn tasks
- [ ] Implement `AuditLogger::emit(event: SecurityEvent)` that writes structured JSON to `tracing`
- [ ] `SecurityEvent` schema: `actor`, `event_type`, `outcome`, `reason_code`, `request_id`, `timestamp`
- [ ] Redact: raw tokens, auth headers, email, secrets (allow: user_id, reason codes, endpoint, status)
- Story coverage: SEC-003, OBS-001, OBS-002, Pattern SEC-02, SEC-03, O-01

### Step 15 — Structured Logging Setup (`src/app.rs`, `src/main.rs`)
- [ ] Configure `tracing-subscriber` with JSON formatter
- [ ] Inject `x-request-id` header middleware; generate UUID if absent
- [ ] Propagate `request_id` through tracing spans
- [ ] Set log level from `AppConfig::LOG_LEVEL`
- [ ] Never log: raw tokens, auth headers, passwords, email addresses
- Story coverage: OBS-001, SECURITY-03, Pattern SEC-02

### Step 16 — API Router (`src/api/router.rs`)
- [ ] Register all UOW-01 routes with the auth middleware applied to protected routes:
  - `POST /api/v1/tokens` — token issuance (protected: authenticated user)
  - `DELETE /api/v1/tokens/{id}` — token revocation (protected)
- [ ] Register bootstrap route for first-time token issuance:
  - `POST /api/v1/bootstrap/tokens` — bootstrap token issuance (unauthenticated, protected by bootstrap key)
- [ ] Define placeholder router extension points for UOW-02 through UOW-05
- [ ] Expose Prometheus `/metrics` endpoint (unauthenticated; internal-network only)
- Story coverage: US-021, US-022 routing layer

### Step 17 — HTTP Security Headers Middleware
- [ ] Add `tower_http::set_header::SetResponseHeader` layers for:
  - `Content-Security-Policy: default-src 'self'`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`
- Story coverage: SECURITY-04

### Step 18 — Unit Tests (`tests/core/`)
- [ ] `auth_validator_test.rs`: success path, each failure reason code, cache hit/miss,
  fail-closed on DB unavailability
- [ ] `token_lifecycle_test.rs`: issuance, revocation, revoked-token rejection,
  ownership checks, 404 for other-user token
- [ ] `error_mapper_test.rs`: snapshot/contract tests for each error category output shape
  (Firefly-compatible schema assertion)
- [ ] `pagination_test.rs`: metadata computation, empty-result edge case, max-page clamping
- [ ] `decimal_serialization_test.rs` **(PBT scope per MNT-001)**:
  property-based round-trip test using `proptest` crate:
  `forall valid decimal string s: deserialize(serialize(s)) == s`
- [ ] Run DB-backed tests against ephemeral PostgreSQL containers using `testcontainers`; apply `migrations/0001_initial_schema.sql` at test startup so integration-style auth and token tests use the real schema
- Story coverage: MNT-001 (PBT), MNT-002 (unit test pyramid), US-021, US-022

### Step 19 — Docker Compose and Deployment Artifacts (`docker/`)
- [ ] `docker/docker-compose.yml` services: `utopia-api`, `postgres`, `caddy`,
  `pgbackrest`, `prometheus`, `grafana`, `loki`, `promtail`, `node-exporter`, `postgres-exporter`
- [ ] Pin all image versions (no `latest` tags)
- [ ] Treat the observability stack (`prometheus`, `grafana`, `loki`, `promtail`, exporters) as optional profiles or overlays so the minimal early-dev stack can run without them
- [ ] Two Docker networks: `edge` (caddy only), `internal` (all other services)
- [ ] PostgreSQL volume and backup volume on named encrypted-path mounts
- [ ] Environment variable injection pattern; no secrets in compose file
- [ ] `docker/caddy/Caddyfile`: TLS auto, reverse proxy to `utopia-api`, access logs,
  HTTP security headers for HTML endpoints
- [ ] `docker/prometheus/prometheus.yml`: scrape targets for `utopia-api`, `node-exporter`,
  `postgres-exporter`; alert rules file reference
- [ ] `docker/grafana/`: provisioning directory stubs for datasources and dashboards
- [ ] `docker/loki/loki-config.yml`: retention set to 90 days
- [ ] `docker/promtail/promtail-config.yml`: ship `utopia-api` and `caddy` container logs to Loki
- Story coverage: SECURITY-01 through SECURITY-14 infrastructure layer, deployment criteria

### Step 20 — Placeholder Module Stubs (`src/modules/mod.rs`)
- [ ] Create `src/modules/mod.rs` declaring `pub mod accounts;`, `pub mod transactions;`,
  `pub mod budgets;`, `pub mod metadata;` with empty module files
- [ ] Ensures the codebase compiles cleanly before UOW-02 starts
- Story coverage: structural prerequisite for UOW-02 through UOW-05

### Step 21 — Code Documentation Summary (`aidlc-docs/construction/core-foundation/code/code-summary.md`)
- [ ] Write markdown summary of:
  - Module structure and file inventory
  - Interface contracts exposed for downstream units
  - Environment variable reference table
  - Run instructions (`cargo build`, `cargo test`, `docker compose up`)
  - Argon2id parameter policy, including how hash-parameter changes affect existing tokens and the recommended response (for example, reissue tokens or perform a staged migration when hashing policy changes)
- Story coverage: MNT-003 (documentation requirements)

## Execution Rules
- Execute steps in order; do not skip or re-order.
- Mark each step `[x]` immediately after completion.
- Mark associated stories `[x]` in this document when their code is generated.
- Never place application code in `aidlc-docs/`.
- Use parameterized queries for all database operations.
- Pin all dependency versions; no `latest` Docker image tags.

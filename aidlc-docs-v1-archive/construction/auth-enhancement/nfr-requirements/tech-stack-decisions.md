# Tech Stack Decisions - Auth Enhancement (UOW-04 / US-021, US-022)

## Context
The auth enhancement unit introduces rate limiting on the bootstrap token endpoint and expands integration test coverage. These decisions build on the existing tech stack established in UOW-01 Core Foundation.

## Existing Stack (Reconfirmed)

| Concern | Decision | Rationale |
|---------|----------|-----------|
| Runtime | Rust (stable) | Already established; no change |
| Web framework | Axum | Already established; middleware stack compatible |
| Database | PostgreSQL via SQLx | Already established; auth data persisted |
| Token hashing | Argon2id | Already established (SEC-001) |
| Structured logging | `tracing` + JSON subscriber | Already established (OBS-001) |
| Metrics | `metrics` + Prometheus exporter | Already established (OBS-002) |
| Test framework | `tokio::test` + `sqlx::test` | Already established |
| Integration tests | `testcontainers` for Docker-backed PostgreSQL | Already established |

## New Decisions for This Unit

### TD-001: Rate Limiting Implementation

**Decision**: Custom in-memory fixed-window counter per IP, implemented as an Axum middleware or service-layer guard.

**Alternatives considered**:
- `tower-governor`: Adds a dependency; fixed-window semantics are simple enough to implement directly; avoids version conflicts.
- Redis-backed rate limiter: Overkill for single-node deployment; introduces an external dependency that contradicts the simplicity goal.
- No rate limiting: Violates security best practices for token bootstrap endpoints.

**Rationale**: 
- Simple fixed-window counter is sufficient for the bootstrap token endpoint's threat model (brute-force prevention).
- In-memory storage keeps the deployment simple (no Redis dependency).
- Per the NFR answers, rate-limit scope is `POST /api/v1/bootstrap/tokens` only; a targeted implementation is more appropriate than a global middleware.

**Implementation notes**:
- Use `std::collections::HashMap<String, Vec<Instant>>` behind an `Arc<RwLock<>>` for thread-safe access.
- Cleanup stale entries periodically (every 60 seconds) to prevent unbounded memory growth.
- Clock source: `std::time::Instant` (monotonic, immune to system clock skew).

### TD-002: Rate Limit Configuration

**Decision**: Environment variables `BOOTSTRAP_RATE_LIMIT_REQUESTS` and `BOOTSTRAP_RATE_LIMIT_WINDOW_SECS`, loaded via the existing `config.rs` pattern.

**Alternatives considered**:
- Hard-coded defaults only: Rejected per NFR-3 answer B (fully configurable).
- Runtime-reloadable config: Rejected per NFR-3 answer B (env vars only, no runtime reload).

**Rationale**: 
- Consistent with existing config pattern (`APP_STRICT_SSL`, database URLs, etc.).
- Environment variables are the standard for 12-factor self-hosted apps.
- No file-watcher or config-reload complexity needed for this scope.

### TD-003: HTTP 429 Response Format

**Decision**: Firefly-III compatible error body with `Retry-After` header.

```json
{
  "message": "Too many requests. Please retry after N seconds.",
  "exception": "RateLimitExceededException"
}
```

**Rationale**: 
- Firefly-III client apps may parse error bodies; maintaining format consistency ensures graceful handling.
- `Retry-After` header is RFC 6585 compliant and enables well-behaved clients to back off.

### TD-004: Integration Test Infrastructure

**Decision**: Extend the existing `tests/` module pattern with a new `tests/auth_integration_test.rs` file.

**Alternatives considered**:
- Separate integration test crate: Adds build complexity; existing pattern works well.
- Mock HTTP server: Doesn't test the real middleware stack.

**Rationale**:
- Consistent with the existing test structure (`tests/accounts_api_test.rs`, `tests/transactions_api_test.rs`).
- Uses the same `testcontainers` infrastructure for Docker-backed PostgreSQL.
- Tests the full Axum router stack including middleware.

### TD-005: Property-Based Testing

**Decision**: Add `proptest`-based tests to `tests/core/token_lifecycle_test.rs` (token format validation) and `tests/core/error_mapper_test.rs` (auth error serialization).

**Alternatives considered**:
- Separate PBT test file: Fragments related test logic; colocation is clearer.
- `quickcheck` instead of `proptest`: `proptest` is more widely used in the Rust ecosystem and integrates better with `tokio::test`.

**Rationale**:
- Consistent with the existing PBT scope (decimal serialization in `tests/core/decimal_serialization_test.rs`).
- Extends Partial enforcement to cover auth-specific pure functions.

## Dependencies (No Changes)

No new crate dependencies are introduced by this unit:
- Rate limiting uses only `std` collections and `tokio::sync`.
- Integration tests reuse existing `reqwest`, `serde_json`, `testcontainers`.
- PBT tests reuse existing `proptest` dev-dependency.

## Constraints

- No external services (Redis, etc.) introduced.
- No new crate dependencies in `Cargo.toml`.
- Implementation stays within `src/core/auth/`, `src/api/`, `src/config.rs`, and `tests/`.

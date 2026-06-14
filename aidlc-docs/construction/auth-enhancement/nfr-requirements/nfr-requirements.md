# NFR Requirements - Auth Enhancement (UOW-04 / US-021, US-022)

## Scope
This document defines non-functional requirements for the auth enhancement unit, covering rate limiting on the bootstrap token endpoint, integration tests for authentication flows, and operational observability additions.

## Baseline Profile
- Deployment profile: self-hosted, single node (extends existing Core Foundation topology).
- Availability posture: best-effort operations (no formal SLA commitment).
- Extension constraints in force:
  - Security Baseline: Enabled (blocking).
  - Property-Based Testing: Partial enforcement.

## Performance Requirements

### PRF-004 Rate Limit Overhead
- Target: Rate-limit check adds < 1ms p99 latency to the bootstrap token endpoint under normal load.
- Rationale: Per-IP fixed-window check should be an in-memory counter lookup with negligible overhead.

### PRF-005 Rate Limit Response Time
- Target: HTTP 429 response returned within 5ms of detection (no I/O path).
- Rationale: Fast rejection prevents resource consumption by rate-limited clients.

## Security Requirements

### SEC-005 Rate Limiting on Bootstrap Endpoint
- A simple per-IP fixed-window rate limiting mechanism MUST be applied on `POST /api/v1/bootstrap/tokens`.
- Default window: 5 requests per 60 seconds per IP.
- Window size and request limit MUST be configurable via environment variables (`BOOTSTRAP_RATE_LIMIT_REQUESTS`, `BOOTSTRAP_RATE_LIMIT_WINDOW_SECS`).
- When the limit is exceeded, the endpoint MUST return HTTP 429 (Too Many Requests) with a `Retry-After` header and a Firefly-III compatible error body.
- Rate limit scope: bootstrap token endpoint only; other endpoints are not affected.

**Verification**:
- Rate limit counter is scoped per source IP address.
- Fixed-window implementation uses monotonic real-time clock (not system clock) to avoid clock-skew bypass.
- Configuration is injected from environment at startup; no runtime reload required.

### SEC-006 Rate Limit Audit Logging
- Every rate limit hit MUST be logged as a security event via the existing structured audit logger.
- Log dimensions MUST include:
  - timestamp
  - source_ip
  - endpoint
  - reason_code (`rate_limit_exceeded`)
  - window_requests (current count)
  - window_limit (configured limit)
- Audit log entries MUST follow the same JSON structured format as other security events (see SEC-003).

### SEC-007 Rate Limit Metrics
- A Prometheus counter for rate-limited requests MUST be added.
- Metric name: `utopia_rate_limited_requests_total`.
- Labels: `endpoint` (e.g., `bootstrap_tokens`), `reason` (e.g., `window_exceeded`).

## Reliability Requirements

### REL-004 Rate Limit Isolation
- Rate limit state (per-IP counters) MUST NOT persist across service restarts.
- Rate limiting failures (e.g., counter overflow) MUST fail open — allow the request and log a warning.
- The rate limit mechanism MUST NOT introduce a single point of failure that blocks bootstrap token issuance.

## Observability Requirements

### OBS-004 Rate Limit Observability (extension of OBS-001/OBS-002)
- Rate-limited requests MUST produce structured log entries (per SEC-006).
- Rate-limited requests MUST increment the Prometheus counter (per SEC-007).
- The existing alert `auth failure rate > 5% for 10 minutes` (OBS-003) also catches rate-limit-induced auth rejections.

## Testability Requirements

### TST-001 Integration Test Scope
- Integration tests MUST cover:
  - Full bootstrap → token issuance → authorized request → token revocation cycle via HTTP.
  - HTTP 401 response for each protected endpoint category (accounts, transactions, budgets, metadata) when no token is provided.
  - Rate limit enforcement: send requests exceeding the configured window limit and verify HTTP 429 with Retry-After header and compatible error body.

### TST-002 Property-Based Testing Scope (Partial Enforcement)
- PBT MUST cover:
  - Token string format validation (generated token strings round-trip through format validation).
  - Authentication error serialization (AuthError variants round-trip through JSON response serialization).
- Other auth paths remain under example-based tests.

## Maintainability Requirements

### MNT-004 Configuration Documentation
- Rate limit environment variables MUST be documented in `.env.example` and the project README.
- Token management endpoints MUST be documented in the API documentation (openapi.yaml or equivalent).

## Constraint Summary
- Rate limiting is lightweight, in-memory, and stateless across restarts.
- No external dependencies (e.g., Redis) are introduced for rate limiting.
- Integration tests use the existing test infrastructure (testcontainers for Docker-backed PostgreSQL).
- PBT scope is limited to pure data transformations as defined by the Partial enforcement mode.

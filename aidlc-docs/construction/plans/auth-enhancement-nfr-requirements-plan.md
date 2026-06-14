# Auth Enhancement — NFR Requirements Plan (US-021 / US-022)

## Background
This plan covers the NFR assessment for implementing rate limiting on the bootstrap token endpoint, adding integration tests, and documenting the token management endpoints.

**Existing NFR Context**:
- **Security Baseline**: Already enforced (opted-in during initial Requirements Analysis)
- **Property-Based Testing**: Partial enforcement (pure functions + serialization round-trips)
- **Tech Stack**: Rust, Axum, SQLx, PostgreSQL (already established)

## NFR Assessment Questions

### Question NFR-1: Rate Limiting Strategy
What rate-limiting approach should be used for the bootstrap token endpoint (`POST /api/v1/bootstrap/tokens`)?

A) Simple per-IP fixed-window rate limiting (e.g., 5 requests per minute per IP) — no external dependencies

B) Token bucket algorithm per IP with configurable refill rate and burst capacity

C) Use a middleware-based approach (e.g., `tower-governor` or Axum middleware layer) for consistent rate limiting across all endpoints

X) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question NFR-2: Rate Limit Scope
Should rate limiting be applied to other endpoints as well?

A) No — only the bootstrap token endpoint needs rate limiting

B) Yes — apply to all auth-related endpoints (tokens, bootstrap) as a defensive measure

C) Yes — apply globally to all API endpoints

X) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question NFR-3: Rate Limit Configuration
How should rate limit parameters be configured?

A) Hard-coded defaults with environment variable overrides

B) Fully configurable via environment variables (requests_per_window, window_seconds, burst_size)

C) Use runtime configuration reloadable without restart

X) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question NFR-4: Rate Limit Response
What should happen when the rate limit is exceeded?

A) Return HTTP 429 (Too Many Requests) with a Retry-After header and a Firefly-III compatible error body

B) Return HTTP 429 with just a standard error message (no Retry-After)

C) Log the violation and return HTTP 429 while continuing to serve other requests

X) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question NFR-5: Integration Test Scope
What scope of integration tests should be added for authentication flows?

A) Test the full bootstrap → token issuance → authorized request → token revocation cycle via HTTP

B) Same as A, plus test 401 for each protected endpoint category (accounts, transactions, etc.)

C) Same as B, plus test rate limit enforcement on bootstrap endpoint

X) Other (please describe after [Answer]: tag below)

[Answer]: C

### Question NFR-6: Property-Based Testing Scope
Given the "Partial" PBT enforcement, which areas should PBT cover for this work?

A) Token string format validation (format round-trip)

B) Authentication error serialization (AuthError → JSON response round-trip)

C) Both A and B

X) Other (please describe after [Answer]: tag below)

[Answer]: C

### Question NFR-7: Audit Logging Enhancement
Should the rate limiting enforcement be logged via the existing audit logger?

A) Yes — log rate limit hits as security events with actor IP and endpoint info

B) No — rate limiting is operational, not a security event

C) Yes — log only for warning thresholds, not every individual rate-limited request

X) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question NFR-8: Metrics for Rate Limiting
Should Prometheus metrics be added for rate limiting?

A) Yes — add counter for rate-limited requests with labels (endpoint, reason)

B) No — existing metrics are sufficient

C) Yes — add both counter and histogram (for time-to-limit)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

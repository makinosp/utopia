# NFR Design Patterns - Auth Enhancement (UOW-04 / US-021, US-022)

## Overview
This document describes the non-functional design patterns applied to the authentication enhancement unit to meet the requirements for rate limiting, observability, and testability.

## Applied Patterns

### 1. Rate Limiting Pattern: Fixed-Window Counter
**Requirement**: SEC-005 (Rate Limiting on Bootstrap Endpoint)
**Pattern**: Fixed-Window Counter
- **Mechanism**: A `HashMap` stores the request count and the start time of the current window for each source IP.
- **Logic**: 
  - If `now - window_start > window_size`, reset count to 1 and update `window_start`.
  - Otherwise, increment count. If `count > limit`, reject with HTTP 429.
- **Rationale**: Simple to implement, low memory overhead, and sufficient for preventing brute-force attacks on the bootstrap endpoint.

### 2. Concurrency Pattern: Read-Write Lock (RwLock)
**Requirement**: PRF-004 (Rate Limit Overhead)
**Pattern**: `Arc<RwLock<T>>`
- **Mechanism**: Use a `RwLock` to protect the shared rate limit state.
- **Logic**: 
  - Use `.read()` for the initial check to allow concurrent requests from different IPs.
  - Use `.write()` only when updating the counter or resetting the window.
- **Rationale**: Minimizes lock contention for the majority of requests, ensuring the < 1ms p99 latency target.

### 3. Resource Management Pattern: Hybrid Eviction
**Requirement**: REL-004 (Rate Limit Isolation / Memory Growth)
**Pattern**: Lazy Eviction + Periodic Background Cleanup
- **Mechanism**: 
  - **Lazy**: When a request arrives, if the window has expired, the entry is reset.
  - **Periodic**: A `tokio::spawn` background task runs every 60 seconds to remove entries that haven't been accessed for > 2x the window size.
- **Rationale**: Prevents unbounded memory growth from one-time attackers while keeping the hot path fast.

### 4. Error Integration Pattern: Unified Error Mapping
**Requirement**: SEC-005 (HTTP 429 Response)
**Pattern**: Error Enum Extension
- **Mechanism**: Add `RateLimitExceeded` variant to the existing `AuthError` enum in `src/core/auth/`.
- **Logic**: The rate limiter returns this error, which is then processed by the existing `error_mapping` layer to produce a Firefly-III compatible JSON response.
- **Rationale**: Ensures consistent error formats across the entire API.

### 5. Observability Pattern: Structured Security Events
**Requirement**: SEC-006 (Audit Logging), SEC-007 (Metrics)
**Pattern**: Structured Event Emission
- **Mechanism**: 
  - **Logging**: Emit `tracing::warn!` events with structured fields (IP, endpoint, reason).
  - **Metrics**: Increment a Prometheus counter `utopia_rate_limited_requests_total` with labels.
- **Rationale**: Integrates seamlessly with the existing observability stack (JSON logs + Prometheus).

### 6. Testing Pattern: Integration Cycle & PBT
**Requirement**: TST-001 (Integration Scope), TST-002 (PBT Scope)
**Pattern**: Full-Cycle Integration + Targeted PBT
- **Mechanism**: 
  - **Integration**: Use `testcontainers` to run a real PostgreSQL instance and test the full HTTP request/response cycle.
  - **PBT**: Use `proptest` to verify that any generated token string can be successfully validated (round-trip) and that any `AuthError` can be serialized/deserialized without loss.
- **Rationale**: Ensures high confidence in the security-critical auth path.

## Security Baseline Compliance
- **SEC-001/02**: No changes to encryption or network intermediaries.
- **SEC-003**: Rate limit hits are now explicitly included in the audit log as security events.
- **SEC-004**: 429 responses do not leak information about user existence.

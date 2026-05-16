# NFR Design Patterns - Core Foundation (UOW-01)

## Scope and Design Intent
This document translates approved NFR requirements into concrete design patterns for resilience, performance, scalability, security, and observability.

## Selected Pattern Decisions (from approved answers)
- Resilience on auth store failure: fail closed, no retries in request path, emit telemetry.
- Circuit breaker: not introduced in initial stage.
- Cache strategy: positive-only in-memory short TTL cache with explicit invalidation on revoke/secret rotation.
- Argon2id tuning: environment-configurable with startup validation and minimum safe bounds.
- Redaction policy: redact all secrets and PII except stable `user_id`.
- Metrics partition: service-level + auth-specific namespace.
- Scaling posture: explicit read/write repository interfaces now; no replica routing yet.
- Alert severity: warning/critical thresholds with escalation windows.
- PBT scope: isolated to serialization helpers for this unit.

## Resilience Patterns

### Pattern R-01: Fail-Closed Authentication Path
- If persistence for token validation is unavailable, authentication fails immediately.
- No in-request retries are executed for token lookup or user resolution.
- Response path remains deterministic through centralized error mapping.

Design rationale:
- Preserves security posture under dependency failure.
- Avoids request amplification and latency spikes from retry storms.

### Pattern R-02: Telemetry-First Failure Handling
- Every auth-store failure emits:
  - structured error log event
  - `auth_dependency_failure_total` metric increment
  - alert evaluation signal

Design rationale:
- Reliability is achieved through fast failure and observability, not hidden retries.

## Performance Patterns

### Pattern P-01: Hot Path Cache for Positive Token Lookups
- Use a local in-memory cache for successful token validation results only.
- Short TTL (recommended 15-60 seconds) to reduce repeated persistence lookups.
- Cache key: token hash fingerprint (never raw token).

### Pattern P-02: Explicit Invalidation Rules
Invalidate positive cache entries when:
- token is revoked
- token-owner block status changes
- secret/key rotation event invalidates hash assumptions

Design rationale:
- Maintains source-of-truth correctness while lowering latency on repeated requests.

## Scalability Patterns

### Pattern S-01: Read/Write Repository Segregation (Design Seam)
- Define separate read and write repository interfaces now.
- Keep routing implementation single-node in this phase.
- Introduce replica routing only in later infrastructure phase when needed.

Design rationale:
- Preserves migration path to read replicas without premature complexity.

### Pattern S-02: No External Cache Dependency in Initial Scope
- Cache remains process-local.
- External shared cache is deferred until scale triggers are met.

## Security Patterns

### Pattern SEC-01: Config-Validated Argon2id Profile
- Argon2id parameters are loaded from environment variables.
- Startup validation enforces minimum secure bounds.
- Invalid bounds fail startup.

Recommended minimum bounds:
- memory_cost >= 65536 KiB
- time_cost >= 3
- parallelism >= 1

### Pattern SEC-02: Structured Redaction Policy
Redact from logs/metrics:
- raw tokens
- authorization headers
- key material and secrets
- user email and other direct identifiers

Allow:
- stable `user_id`
- reason codes
- endpoint and status metadata

### Pattern SEC-03: Security Event Audit Envelope
Security-sensitive events include:
- actor (`user_id` if authenticated)
- event_type
- outcome
- reason_code
- request_id
- timestamp (UTC)

## Observability and Alerting Patterns

### Pattern O-01: Dual Namespace Metrics
- Service namespace: request volume, 5xx rates, latency summary
- Auth namespace: validation latency histogram, failure counters by reason, cache hit/miss counters

### Pattern O-02: Auth Failure Spike Alert Ladder
- Warning: auth failure rate > 5 percent for 10 minutes
- Critical: warning condition persists for 15 additional minutes OR repeats in 3 consecutive evaluation windows

Escalation behavior:
- Warning notifies maintainers.
- Critical triggers escalation policy documented in runbook.

## Quality and Testing Patterns

### Pattern Q-01: PBT Scope Guardrail
- Property-based tests are limited to serialization helper round-trips for this unit.
- Additional property domains (pagination invariants/auth mapping) remain out of scope until explicitly elevated.

### Pattern Q-02: Contract Snapshot Baseline
- Keep stable snapshot tests for Firefly-compatible error/pagination envelopes to detect accidental contract drift.

## Deferred Patterns
- Circuit breaker lifecycle management (deferred)
- Distributed tracing in this unit (deferred)
- External shared cache and cache coherence protocols (deferred)
- Multi-node routing and replica-lag-aware reads (deferred)

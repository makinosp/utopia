# NFR Design Plan - Core Foundation (UOW-01)

## Objective
Translate approved NFR requirements into concrete non-functional design patterns and logical components for Core Foundation.

## Context Inputs
- Unit: UOW-01 Core Foundation
- Source artifacts:
  - aidlc-docs/construction/core-foundation/nfr-requirements/nfr-requirements.md
  - aidlc-docs/construction/core-foundation/nfr-requirements/tech-stack-decisions.md
  - aidlc-docs/construction/core-foundation/functional-design/business-logic-model.md

## NFR Design Checklist
- [x] Define resilience patterns for auth and persistence failure paths
- [x] Define scalability patterns aligned with single-node baseline and replica-ready evolution
- [x] Define performance patterns for p95 <= 100ms token validation
- [x] Define security patterns for Argon2id, secrets rotation, and auditability
- [x] Define observability patterns (logs, metrics, alerting thresholds)
- [x] Define logical components and integration boundaries
- [x] Generate `nfr-design-patterns.md`
- [x] Generate `logical-components.md`
- [x] Validate design consistency against approved NFR requirements
- [x] Request approval to proceed to Infrastructure Design

## Planning Questions

Please fill all `[Answer]:` fields.

## Question 1
What resilience policy should be used when database access fails during token validation?

A) Fail closed immediately (401/500 as mapped), no retries in request path
B) Retry once with short backoff, then fail closed
C) Retry up to 3 times with exponential backoff, then fail closed
D) Circuit breaker + fallback cache for previously validated tokens
X) Other (please describe after [Answer]: tag below)

[Answer]: A - Fail closed without request-path retries, but still emit structured logs, metrics, and alert events for the failure.

## Question 2
How should circuit breaker behavior be designed for the auth/persistence boundary?

A) No circuit breaker in initial stage
B) Passive breaker (monitor only, no traffic shedding)
C) Active breaker with open/half-open/closed states and short recovery probes
D) Active breaker with per-endpoint thresholds
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 3
Which cache strategy should be applied to token validation in this stage?

A) No cache; always validate against persistent store
B) In-memory short TTL cache for positive token lookups only
C) In-memory TTL cache for positive and negative lookups
D) Shared external cache (Redis) required from day one
X) Other (please describe after [Answer]: tag below)

[Answer]: X - Positive-only in-memory short TTL cache with explicit invalidation on token revocation or secret rotation; persistence remains the source of truth on cache miss.

## Question 4
How should Argon2id parameter tuning be handled for performance/security balance?

A) Fixed conservative defaults only
B) Environment-configurable with startup validation and safe minimum bounds
C) Environment-configurable + runtime hot reload
D) Auto-tune at startup based on host benchmarks
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 5
What log redaction policy should be enforced?

A) Redact raw tokens only
B) Redact all secrets and credential-like values (tokens, auth headers, key material)
C) Redact secrets + user-identifying fields except stable user_id
D) Redact everything sensitive and store only aggregate event counters
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 6
How should metrics and alerts be partitioned for operations?

A) Global service-level metrics only
B) Service-level + auth-specific metric namespace
C) Service-level + auth + endpoint-level metrics
D) Full high-cardinality metrics with per-user labels
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 7
What scaling design posture should be documented for read replicas?

A) Mention as future option only (no explicit design seams)
B) Define explicit read/write repository interfaces now, no replica routing yet
C) Implement read routing abstraction now with feature flag
D) Full read replica routing design including lag-aware reads
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 8
How should alert severity mapping be designed for auth failure spikes?

A) Single warning threshold only
B) Warning + critical thresholds with escalation windows
C) Multi-level thresholds with auto-silence during maintenance windows
D) No alerts; rely on manual dashboard monitoring
X) Other (please describe after [Answer]: tag below)

[Answer]: X - Warning when auth failure rate exceeds 5 percent for 10 minutes; critical when the condition persists for 15 minutes or repeats across three consecutive evaluation windows; document escalation windows in the design artifact.

## Question 9
What property-based test integration pattern should be reflected in NFR design?

A) Keep PBT isolated to serialization helpers only
B) Integrate PBT in shared test utilities for all core value-object round trips
C) Integrate PBT in CI gating for selected core modules
D) Defer PBT integration design to code generation stage
X) Other (please describe after [Answer]: tag below)

[Answer]: A - Keep PBT scoped to serialization helpers only in this unit, matching the approved partial enforcement for decimal round-trip properties.

## Question 10
Which logical component boundary should be emphasized for Core Foundation?

A) Auth middleware as monolithic cross-cutting component
B) Separate components: Auth Validator, Error Mapper, Metrics Emitter, Audit Logger
C) Separate components + Policy Engine for authorization and redaction rules
D) Event-driven internal components with async message handoff
X) Other (please describe after [Answer]: tag below)

[Answer]: X - Use in-process logical subcomponents for Auth Validator, Error Mapper, Metrics, and Audit concerns, composed behind a single auth middleware facade rather than separately deployed components.

# NFR Requirements - Core Foundation (UOW-01)

## Scope
This document defines non-functional requirements for Core Foundation covering authentication, shared compatibility contracts, and cross-cutting security controls.

## Baseline Profile
- Deployment profile: self-hosted, single node primary with optional read replicas later.
- Availability posture: best-effort operations (no formal SLA commitment).
- Extension constraints in force:
  - Security Baseline: Enabled (blocking).
  - Property-Based Testing: Partial enforcement.

## Performance Requirements

### PRF-001 Token Validation Latency
- Target: p95 <= 100ms for token validation on protected endpoints under baseline load.
- Measurement window: 5-minute rolling window.
- Excludes upstream reverse proxy latency.

### PRF-002 Sustained Throughput
- Target: 100 authenticated requests/second sustained for baseline deployment.
- Burst tolerance: up to 150 requests/second for short intervals (<= 60 seconds) without error-rate regression above threshold.

### PRF-003 Error Budget for Performance Degradation
- If p95 token validation latency exceeds 100ms for 3 consecutive 5-minute windows, system must emit warning-level operational alerts.

## Scalability Requirements

### SCL-001 Baseline Topology
- Initial sizing assumes single-node service deployment.
- Architecture must preserve a migration path to add read replicas for read-heavy operations.

### SCL-002 Scale Trigger
- Scale-out review is required when either condition is met:
  - Sustained throughput exceeds 80 requests/second for 7 consecutive days.
  - CPU utilization exceeds 70 percent during peak periods for 7 consecutive days.

## Availability and Recovery Requirements

### AVA-001 Availability Objective
- Service is operated on a best-effort basis.
- No monthly SLA/SLO commitment is required for personal OSS deployment.

### AVA-002 Recovery Objective
- Formal RTO/RPO targets are not required for this unit in initial scope.
- Minimum expectation: documented manual recovery runbook for token/auth storage restoration.

## Security Requirements

### SEC-001 Token Hashing
- Token hashing algorithm: Argon2id.
- Raw tokens must never be persisted.
- Hash parameters must be configurable and documented.

### SEC-002 Secrets Management
- Secrets are injected via environment variables.
- A documented rotation schedule is mandatory.
- Rotation cadence must be at least quarterly for signing/secret values.

### SEC-003 Audit Logging Scope
- Extended audit logging is required for security-sensitive actions:
  - token issuance
  - token revocation
  - authentication failures
  - blocked-user access attempts
  - authorization denials
  - security-relevant configuration changes

### SEC-004 Data Exposure Control
- Authentication and authorization failures must avoid leaking cross-user resource existence.
- Ownership failures continue to map to resource-not-found semantics where defined by functional rules.

## Reliability Requirements

### REL-001 Authentication Path Fault Handling
- Authentication failure scenarios must be deterministic and mapped to documented error payloads.
- Failure classification must remain stable across releases.

### REL-002 Transaction Boundary Integrity
- One mutating request must map to one transaction boundary.
- Partial write visibility is prohibited.

### REL-003 Dependency Failure Behavior
- If persistence is unavailable, service must fail closed for protected operations and emit structured error and alert events.

## Observability Requirements

### OBS-001 Structured Logging
- JSON structured logs required for all protected endpoint requests and auth outcomes.
- Required dimensions:
  - timestamp
  - request_id
  - user_id (if authenticated)
  - endpoint
  - status_code
  - auth_outcome

### OBS-002 Metrics
- Required metrics:
  - auth_validation_latency_ms (histogram)
  - authenticated_requests_total (counter)
  - auth_failures_total by reason_code (counter)
  - token_issue_total and token_revoke_total (counter)
  - http_5xx_total (counter)

### OBS-003 Alerting Baseline
- Alert when:
  - auth failure rate > 5 percent for 10 minutes
  - http_5xx rate > 1 percent for 10 minutes
  - p95 auth validation latency > 100ms for 15 minutes

## Maintainability and Quality Requirements

### MNT-001 Property-Based Testing Scope (Partial)
- Enforced PBT scope for this unit:
  - decimal serialization round-trip properties only
- Other concerns (pagination invariants, auth error mapping properties) are example-based in this phase.

### MNT-002 Test Pyramid Baseline
- Unit tests required for:
  - auth validation path
  - error mapping path
  - token lifecycle transitions
- Contract tests required for Firefly-compatible error and pagination structures.

### MNT-003 Documentation Requirements
- Security configuration and secret rotation procedures must be documented before production-like deployment.

## Constraint Summary
- Self-hosted personal OSS prioritizes simplicity over formal enterprise HA commitments.
- Security controls remain strict despite best-effort availability posture.
- Future scale path is preserved but not implemented upfront.

# NFR Requirements Plan - Core Foundation (UOW-01)

## Objective
Define non-functional requirements and technology decisions for Core Foundation with emphasis on security baseline enforcement and reliability for authentication and cross-cutting services.

## Context Inputs
- Unit: UOW-01 Core Foundation
- Functional design artifacts:
  - aidlc-docs/construction/core-foundation/functional-design/business-logic-model.md
  - aidlc-docs/construction/core-foundation/functional-design/business-rules.md
  - aidlc-docs/construction/core-foundation/functional-design/domain-entities.md
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## NFR Planning Checklist
- [x] Analyze security risk profile for authentication and token handling
- [x] Define performance targets for auth-critical request path
- [x] Define scalability targets and trigger points
- [x] Define reliability and failure-handling requirements
- [x] Define availability and recovery expectations
- [x] Define observability and operational monitoring requirements
- [x] Define maintainability and testing quality gates
- [x] Define technology constraints and stack decisions
- [x] Generate NFR artifacts (`nfr-requirements.md`, `tech-stack-decisions.md`)
- [x] Request approval to proceed to NFR Design

## Planning Questions

Please fill all `[Answer]:` fields.

## Question 1
What availability target should be set for the authentication and shared core endpoints?

A) Best-effort for self-hosted use (no strict SLA)
B) 99.0% monthly availability target
C) 99.9% monthly availability target
D) 99.95% monthly availability target
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 2
What p95 latency target should be required for token validation on protected endpoints?

A) <= 200ms
B) <= 100ms
C) <= 50ms
D) <= 20ms
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 3
What baseline throughput target should be used for initial self-hosted deployments?

A) 50 authenticated requests/second sustained
B) 100 authenticated requests/second sustained
C) 250 authenticated requests/second sustained
D) 500 authenticated requests/second sustained
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 4
Which token hashing strategy should be required?

A) SHA-256 only for API token hashing
B) Argon2id for API token hashing
C) Bcrypt for API token hashing
D) Hybrid: SHA-256 lookup key + slow hash for defense-in-depth
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 5
How should secrets and key material be managed in initial scope?

A) Environment variables only
B) Environment variables with mandatory rotation schedule documentation
C) External secret manager required from day one
D) Config file with encrypted values at rest
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 6
What should be the required audit logging level for security-sensitive events?

A) Minimal: login success/failure only
B) Standard: token issue/revoke + auth failures + blocked-user access attempts
C) Extended: standard + all authorization denials + config changes
D) Comprehensive: extended + request metadata and actor context for all protected requests
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 7
Which incident recovery objective should be targeted for this unit?

A) RTO 24h / RPO 24h
B) RTO 8h / RPO 4h
C) RTO 4h / RPO 1h
D) RTO 1h / RPO 15m
X) Other (please describe after [Answer]: tag below)

[Answer]: X
Personal OSS development does not require formal incident recovery objectives.

## Question 8
What observability baseline should be enforced?

A) Structured logs only
B) Structured logs + metrics (latency, error rate, throughput)
C) Structured logs + metrics + distributed tracing
D) Full observability + SLO burn-rate alerting
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 9
How strict should property-based testing be for this unit in current phase?

A) Apply only to decimal serialization round-trips
B) Apply to serialization + pagination metadata invariants
C) Apply to serialization + invariants + auth error mapping properties
D) Skip PBT for this unit; use example-based tests only
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 10
Which deployment baseline should be assumed for NFR sizing?

A) Single-node deployment only
B) Single-node with optional read replicas later
C) Active-passive two-node deployment
D) Active-active multi-node deployment
X) Other (please describe after [Answer]: tag below)

[Answer]: B

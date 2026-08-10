# Tech Stack Decisions - Core Foundation (UOW-01)

## Decision Summary

| Area | Decision | Rationale |
|---|---|---|
| Language/runtime | Rust | Existing project direction and memory-safe baseline.
| API framework | Axum (planned) | Good async ecosystem alignment and middleware composition for auth path.
| Password/token hashing | Argon2id | Strong modern resistance to GPU/ASIC brute-force.
| Storage | PostgreSQL | Existing architecture decision and transactional consistency requirements.
| Serialization | Serde + strict DTO mapping layer | Supports Firefly-compatible contracts with explicit transformations.
| Decimal arithmetic | rust_decimal | Preserves decimal precision for monetary values.
| Logging | Structured JSON logging (tracing ecosystem) | Supports required observability dimensions and auditability.
| Metrics | Prometheus-compatible metrics exporter | Covers latency/error/throughput NFR observability baseline.
| Secrets source | Environment variables + rotation runbook | Matches initial deployment simplicity while enforcing rotation discipline.
| Deployment baseline | Single node with optional read replicas later | Aligns with selected NFR scaling assumptions.

## Security Decisions

### TS-SEC-001 Argon2id Parameters
- Adopt configurable Argon2id parameters via environment variables.
- Minimum profile recommendation for initial scope:
  - memory_cost: 64 MB
  - time_cost: 3
  - parallelism: 1
- Parameter values may be tuned after baseline performance testing.

### TS-SEC-002 Token Storage Pattern
- Persist only token hash and metadata.
- Raw token displayed once at issuance and never recoverable.

### TS-SEC-003 Audit Trail Strategy
- Emit structured audit events for extended security event set.
- Ensure reason_code tagging for auth failures to support incident diagnosis.

## Performance and Reliability Decisions

### TS-PRF-001 Auth Path Optimization Direction
- Keep auth lookup path index-friendly (hashed token lookup key indexed).
- Minimize per-request allocations in auth middleware.

### TS-REL-001 Transaction Boundary Implementation
- Use centralized transaction manager abstraction for one-request/one-write-transaction policy.
- Avoid ad hoc transaction handling in handlers.

## Observability Decisions

### TS-OBS-001 Logging and Correlation
- Require request_id propagation through middleware and service boundaries.
- Include user_id only after successful authentication to avoid misleading identity attribution.

### TS-OBS-002 Metrics Cardinality Guardrails
- Limit label dimensions for counters/histograms to avoid high-cardinality blowups.
- For auth failures, reason_code is permitted; raw token/user email labels are prohibited.

## Alternatives Considered

### Alternative A: Bcrypt for token hashing
- Rejected: lower memory hardness than Argon2id for this use case.

### Alternative B: External secret manager from day one
- Deferred: increases operational complexity for personal OSS baseline.

### Alternative C: Distributed tracing baseline in this stage
- Deferred: valuable later, but current scope selected logs+metrics as sufficient baseline.

## Follow-up Actions for Next Stage (NFR Design)
- Convert these decisions into explicit component-level patterns:
  - auth middleware contracts
  - audit event schema
  - metrics naming and alert thresholds
  - security configuration profile matrix

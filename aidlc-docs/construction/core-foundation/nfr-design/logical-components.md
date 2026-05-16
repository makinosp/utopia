# Logical Components - Core Foundation (UOW-01)

## Architecture Boundary
Core Foundation remains an in-process composition. Logical subcomponents are separated by interfaces but deployed as one service process behind a single auth middleware facade.

## Component Inventory

| Component | Responsibility | Inputs | Outputs |
|---|---|---|---|
| Auth Middleware Facade | Entry point for protected request auth flow orchestration | HTTP request context | Auth success context or mapped error response |
| Auth Validator | Token extraction, format checks, token/user lookup coordination | Bearer token, repository access | Validated principal or auth failure reason |
| Token Cache | Positive token validation short-TTL cache | Token hash fingerprint, principal snapshot | Cache hit/miss + principal snapshot |
| Error Mapper | Normalize failures to Firefly-compatible payloads | Domain/auth/infra errors | HTTP status + error envelope |
| Metrics Emitter | Emit service/auth metrics and counters | Auth outcomes, timing data | Metrics stream |
| Audit Logger | Emit security event records with redaction | Event payload, context fields | Structured audit log events |
| Config Validator | Startup validation for Argon2id/security config | Environment values | Validated config or startup failure |
| Repository Read Interface | User/token read access seam | Query models + principal context | Domain records |
| Repository Write Interface | Token revoke/update and audit-related writes | Command models | Write result |

## Integration Topology (Text)
1. Request enters Auth Middleware Facade.
2. Facade calls Auth Validator.
3. Auth Validator checks Token Cache first.
4. On cache miss, validator uses Repository Read Interface.
5. Validator returns success principal or failure reason.
6. Facade sends outcomes to Metrics Emitter and Audit Logger.
7. On failure, Error Mapper produces standardized response.
8. On revoke/config-rotation events, Repository Write Interface triggers cache invalidation workflow.

## Component Contracts

### Auth Middleware Facade Contract
- Must not embed persistence logic directly.
- Must ensure every failure path is mapped through Error Mapper.
- Must emit observability hooks for both success and failure.

### Auth Validator Contract
- Must fail closed on dependency failures.
- Must not perform retries in request path.
- Must produce explicit reason code taxonomy.

### Token Cache Contract
- Positive-only cache entries.
- TTL required; hard max TTL configuration enforced.
- Invalidation API required for token revoke and secret rotation events.

### Error Mapper Contract
- Must generate Firefly-compatible `{message, errors}` schema.
- Must preserve deterministic status mapping rules from functional design and NFR requirements.

### Metrics Emitter Contract
- Service-level metrics and auth namespace metrics are separate.
- High-cardinality labels are forbidden (no raw token, no email, no per-user label except bounded `user_id` only where justified).

### Audit Logger Contract
- Redact all secrets and credential-like fields.
- Retain stable `user_id` and reason codes for diagnostics.
- Use UTC timestamps and request correlation IDs.

## Non-Functional Mapping Matrix

| NFR Category | Primary Components | Supporting Components |
|---|---|---|
| Security | Auth Validator, Config Validator, Audit Logger | Error Mapper, Token Cache |
| Performance | Token Cache, Auth Validator | Metrics Emitter |
| Reliability | Auth Validator, Error Mapper | Audit Logger, Metrics Emitter |
| Scalability | Repository Read/Write Interfaces | Token Cache |
| Observability | Metrics Emitter, Audit Logger | Auth Middleware Facade |

## Operational Signals by Component

| Component | Must Emit |
|---|---|
| Auth Validator | auth_success_total, auth_failure_total by reason |
| Token Cache | auth_cache_hit_total, auth_cache_miss_total, invalidation_total |
| Error Mapper | error_mapping_total by category |
| Metrics Emitter | p95 auth latency histogram, 5xx counters |
| Audit Logger | structured security event records |

## Deferred Componentization
- No separate process/service split for subcomponents in this phase.
- No policy-engine component in this phase.
- No async internal event bus for auth path in this phase.

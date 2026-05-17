# Code Summary - Core Foundation (UOW-01)

## Generated Scope
The code generation for UOW-01 implemented authentication, token issuance/revocation, Firefly-compatible error mapping, compatibility primitives, request-scoped transaction handling in services, and deployment artifacts.

## Module Inventory

| Area | Key Files |
|---|---|
| Entry and app wiring | `src/main.rs`, `src/app.rs`, `src/config.rs` |
| Auth core | `src/core/auth/models.rs`, `src/core/auth/error.rs`, `src/core/auth/validator.rs`, `src/core/auth/middleware.rs`, `src/core/auth/service.rs` |
| Auth support | `src/core/auth/cache.rs`, `src/core/auth/metrics.rs`, `src/core/auth/audit_logger.rs` |
| Compatibility | `src/core/compatibility/pagination.rs`, `src/core/compatibility/envelope.rs`, `src/core/compatibility/error_response.rs`, `src/core/compatibility/decimal_amount.rs` |
| Error mapping | `src/core/error_mapping/mapper.rs` |
| Persistence | `src/core/persistence/db.rs`, `src/core/persistence/repository.rs`, `migrations/0001_initial_schema.sql` |
| API layer | `src/api/router.rs`, `src/api/handlers/tokens.rs` |
| Future unit placeholders | `src/modules/accounts.rs`, `src/modules/transactions.rs`, `src/modules/budgets.rs`, `src/modules/metadata.rs` |
| Tests | `tests/core/*.rs` |
| Deployment | `Dockerfile`, `docker/docker-compose.yml`, `docker/**` |

## Exposed Core Contracts for Downstream Units
- `Principal` in `src/core/auth/models.rs`
- Auth middleware insertion and protected route composition in `src/core/auth/middleware.rs`
- Firefly-compatible error response shape in `src/core/compatibility/error_response.rs`
- Pagination metadata envelope in `src/core/compatibility/pagination.rs`
- Repository trait seams in `src/core/persistence/repository.rs`

## Environment Variables

| Variable | Purpose |
|---|---|
| `APP_PORT` | HTTP listening port |
| `LOG_LEVEL` | Tracing level filter |
| `DATABASE_URL` | PostgreSQL connection string (TLS required) |
| `ARGON2_MEMORY_COST` | Argon2id memory cost |
| `ARGON2_TIME_COST` | Argon2id iterations |
| `ARGON2_PARALLELISM` | Argon2id parallelism |
| `TOKEN_CACHE_TTL_SECS` | Positive token cache TTL |
| `NEGATIVE_TOKEN_CACHE_TTL_SECS` | Negative token cache TTL |
| `TOKEN_CACHE_MAX_CAPACITY` | Max cache entries |
| `BOOTSTRAP_KEY` | One-time bootstrap key |
| `BOOTSTRAP_USER_EMAIL` | Bootstrap identity email |

## Run and Test Instructions
```bash
cargo build
cargo test
docker compose -f docker/docker-compose.yml up -d
docker compose -f docker/docker-compose.yml --profile observability up -d
```

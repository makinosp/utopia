# Integration Test Instructions

## Purpose
Validate interactions between middleware, token service, persistence repositories, and migration-enabled PostgreSQL runtime.

## Integration Scenarios

### Scenario 1: Bootstrap Token Issuance Flow
Description:
- Verify POST /api/v1/bootstrap/tokens creates bootstrap user and first PAT only once

Setup:
- Postgres running in compose
- .env configured with BOOTSTRAP_KEY and BOOTSTRAP_USER_EMAIL

Test steps:
1. Start postgres and app containers
2. Call bootstrap endpoint with valid X-Bootstrap-Key
3. Assert 200 response with token payload
4. Call bootstrap endpoint again with same key
5. Assert rejection with bootstrap_key_already_used reason

Expected results:
- First request succeeds
- Second request fails closed

Cleanup:
```bash
docker compose -f docker/docker-compose.yml down -v
```

### Scenario 2: Auth Middleware and Revocation Flow
Description:
- Verify protected route enforcement and token revocation invalidation behavior

Test steps:
1. Issue token for authenticated principal
2. Access protected endpoint with Bearer token
3. Revoke token via DELETE /api/v1/tokens/:id
4. Retry protected request with revoked token

Expected results:
- Pre-revocation call succeeds
- Post-revocation call returns 401 with token_revoked reason

## Setup Integration Environment

### 1. Start Required Services
```bash
docker compose -f docker/docker-compose.yml up -d postgres
```

### 2. Apply Migrations and Run App
```bash
cargo run
```

### 3. Run DB-backed Integration Tests
```bash
cargo test --test db_integration_test -- --ignored
```

## Optional API-level Integration Checks
Example with curl after starting app locally:
```bash
curl -i -X POST http://localhost:3000/api/v1/bootstrap/tokens \
  -H "Content-Type: application/json" \
  -H "X-Bootstrap-Key: ${BOOTSTRAP_KEY}" \
  -d '{"label":"initial-bootstrap"}'
```

## Cleanup
```bash
docker compose -f docker/docker-compose.yml down
```

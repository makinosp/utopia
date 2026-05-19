# Integration Test Instructions

## Purpose
Validate interactions between middleware, token service, persistence repositories, and migration-enabled PostgreSQL runtime.

## Integration Scenarios

### Scenario 1: Bootstrap Token Issuance Flow
Description:
- Verify POST /api/v1/bootstrap/tokens creates bootstrap user and first PAT only once

Setup:
- Postgres running in compose with host port 5432 exposed (see note in Setup Integration Environment)
- `.env` configured with all required variables — see [build-instructions.md](build-instructions.md) for the full list
- `jq` available in PATH (used for JSON extraction)

Test steps:
1. Start Postgres and run the app (see Setup Integration Environment)
2. Call the bootstrap endpoint with a valid `X-Bootstrap-Key` and save the returned token:
```bash
BOOTSTRAP_RESPONSE=$(curl -s -X POST http://localhost:3000/api/v1/bootstrap/tokens \
  -H "Content-Type: application/json" \
  -H "X-Bootstrap-Key: ${BOOTSTRAP_KEY}" \
  -d '{"label":"initial-bootstrap"}')
echo $BOOTSTRAP_RESPONSE
TOKEN_A=$(echo $BOOTSTRAP_RESPONSE | jq -r '.data.token')
```
3. Assert the response is HTTP 200 with a `data.token` field
4. Call the bootstrap endpoint a second time with the same key:
```bash
curl -i -X POST http://localhost:3000/api/v1/bootstrap/tokens \
  -H "Content-Type: application/json" \
  -H "X-Bootstrap-Key: ${BOOTSTRAP_KEY}" \
  -d '{"label":"repeat-bootstrap"}'
```
5. Assert the response is HTTP 401 with `bootstrap_key_already_used` in the message body

Expected results:
- Step 2 returns 200 with token payload
- Step 4 returns 401 with `bootstrap_key_already_used`

Cleanup:
```bash
docker compose -f docker/docker-compose.yml down -v
```

### Scenario 2: Auth Middleware and Revocation Flow
Description:
- Verify protected route enforcement and token revocation invalidation behavior

Setup:
- Scenario 1 completed successfully; `TOKEN_A` is available in the shell session
- App still running
- `jq` available in PATH

Test steps:
1. Issue a second token (`TOKEN_B`) using `TOKEN_A` for authentication:
```bash
TOKEN_B_RESPONSE=$(curl -s -X POST http://localhost:3000/api/v1/tokens \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN_A}" \
  -d '{"label":"test-revoke"}')
TOKEN_B=$(echo $TOKEN_B_RESPONSE | jq -r '.data.token')
TOKEN_B_ID=$(echo $TOKEN_B_RESPONSE | jq -r '.data.id')
```
2. Verify `TOKEN_B` is accepted on a protected endpoint:
```bash
curl -i -X POST http://localhost:3000/api/v1/tokens \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN_B}" \
  -d '{"label":"verify"}'
```
3. Revoke `TOKEN_B` using `TOKEN_A`:
```bash
curl -i -X DELETE http://localhost:3000/api/v1/tokens/${TOKEN_B_ID} \
  -H "Authorization: Bearer ${TOKEN_A}"
```
4. Retry the protected endpoint with the now-revoked `TOKEN_B`:
```bash
curl -i -X POST http://localhost:3000/api/v1/tokens \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN_B}" \
  -d '{"label":"should-fail"}'
```

Expected results:
- Step 2 returns 200 with token payload
- Step 3 returns 204
- Step 4 returns 401 with `token_revoked` in the message body

## Setup Integration Environment

### 1. Start Required Services

> **Note**: The default `docker/docker-compose.yml` does not expose the Postgres port to the host.
> To run `cargo run` on the host machine against the compose Postgres, add the following to the
> `postgres` service in a compose override file (`docker-compose.override.yml`):
> ```yaml
> services:
>   postgres:
>     ports:
>       - "5432:5432"
> ```
> Then start with: `docker compose -f docker/docker-compose.yml -f docker/docker-compose.override.yml up -d postgres`

```bash
docker compose -f docker/docker-compose.yml up -d postgres
```

### 2. Apply Migrations and Run App
```bash
cargo run
```

### 3. Run DB-backed Integration Tests

> **Note**: The current `db_integration_test.rs` verifies that an ephemeral Postgres container
> starts successfully. It does not exercise the business scenarios described above (Scenario 1
> and 2). Use the curl commands in each scenario's test steps for those validations.

```bash
cargo test --test db_integration_test -- --ignored
```

## Cleanup
```bash
docker compose -f docker/docker-compose.yml down -v
```

> The `-v` flag removes the persistent `postgres_data` volume, ensuring bootstrap state is fully
> reset between test runs.

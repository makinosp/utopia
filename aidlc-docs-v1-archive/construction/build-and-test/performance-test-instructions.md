# Performance Test Instructions

## Purpose
Validate auth, accounts, and transactions endpoints against the approved NFR baseline using the UOW-05 k6 compatibility verification suite.

## Performance Targets
- Auth validation latency: p95 <= 100 ms
- Sustained throughput: 100 authenticated requests/second
- Burst throughput: 150 requests/second up to 60 seconds
- Error thresholds:
  - Auth failure rate alert at > 5 percent for 10 minutes
  - HTTP 5xx alert at > 1 percent for 10 minutes

## Test Environment Setup

### 1. Start Runtime
```bash
docker compose -f docker/docker-compose.yml up -d postgres
docker compose -f docker/docker-compose.yml --profile observability up -d prometheus grafana loki promtail node-exporter postgres-exporter
cargo run
```

### 2. Prepare Test Data
```bash
# Load deterministic seed fixtures (UOW-05)
bun run scripts/seed/index.ts
```

### 3. Configure Load Test Parameters
- Duration: 15 minutes
- Ramp-up: 60 seconds
- Virtual users: 100 baseline, 150 burst phase

## UOW-05 Compatibility Verification Suite (k6)

### Run Full Suite
```bash
# Using native k6
export APP_BASE_URL=http://localhost:3000
./k6/run-all.sh

# Using Docker k6 service
docker compose --profile testing up -d
docker compose run --rm k6 run /scripts/k6/auth.ts
docker compose run --rm k6 run /scripts/k6/accounts.ts
docker compose run --rm k6 run /scripts/k6/transactions.ts
```

### Run Individual Test Scripts
```bash
# Auth flow tests (token issuance, revocation, rejection)
k6 run -e BASE_URL=http://localhost:3000 k6/auth.ts

# Accounts API tests (CRUD + pagination + type filter)
k6 run -e BASE_URL=http://localhost:3000 k6/accounts.ts

# Transactions API tests (CRUD + pagination + account filter)
k6 run -e BASE_URL=http://localhost:3000 k6/transactions.ts
```

### Test Coverage
| Script | Validates |
|--------|-----------|
| `k6/auth.ts` | Bootstrap token issuance, authenticated request, unauthenticated rejection, token revocation, revoked token rejection, invalid bootstrap key |
| `k6/accounts.ts` | List (pagination/type filter), get, create, update, delete, verify deletion |
| `k6/transactions.ts` | List (pagination/type filter), get, create, update, delete, verify deletion, list by account |

### k6 Check Functions (Firefly-III Compatibility)
The harness (`k6/harness.ts`) validates:
- List envelope structure (`data` + `meta.pagination`)
- Single envelope structure (`data` with `type`, `id`, `attributes`, `links`)
- Error envelope format (`message` + `errors`)
- 204 No Content responses (DELETE)
- 401 Unauthorized responses
- Pagination consistency (total, count, per_page, current_page, total_pages)

## Result Evaluation
Collect:
- k6 output summary (latency and rate)
- JSON results in `k6-results/` directory
- /metrics values for auth_validation_latency_ms and http_5xx_total
- Grafana dashboard snapshots for test window

Pass criteria:
- p95 auth validation latency <= 100 ms
- sustained >= 100 rps with acceptable error budget
- burst phase does not trigger prolonged 5xx instability
- All Firefly-III envelope checks pass (0 failed checks in k6 output)

## If Performance Fails
1. Inspect cache hit/miss and dependency failure counters
2. Inspect Postgres query latency and connection pool saturation
3. Tune cache TTL and pool settings
4. Re-run same scenario and compare results

## If Compatibility Checks Fail
1. Review k6 check failures in `k6-results/` JSON output
2. Compare actual response structure against `k6/fixtures/` expected schemas
3. Fix API response format in handlers to match Firefly-III contract
4. Re-run affected test script until all checks pass

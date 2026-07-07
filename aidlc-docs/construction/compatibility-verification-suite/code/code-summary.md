# Code Summary — UOW-05: Compatibility Verification Suite

## Overview
This document summarizes all generated files for the Compatibility Verification Suite, a k6-based automated test harness that validates Utopia's API responses against Firefly-III compatibility requirements.

## Generated Files

### Seed Data Generator (`scripts/seed/`)

| File | Purpose |
|------|---------|
| `scripts/seed/package.json` | Dependencies (pg, dotenv) and Bun runtime config |
| `scripts/seed/index.ts` | Entry point: truncates all tables, inserts deterministic fixtures |
| `scripts/seed/types.ts` | Shared TypeScript interfaces (AccountSeed, TransactionSeed, UserSeed) |
| `scripts/seed/accounts.ts` | 6 account fixtures covering asset, cash, expense, revenue, liability types |
| `scripts/seed/transactions.ts` | 5 transaction fixtures (deposit, withdrawal, transfer) |

**Execution**: `bun run scripts/seed/index.ts` (requires `DATABASE_URL` env var)

### k6 Test Fixtures (`k6/fixtures/`)

| File | Purpose |
|------|---------|
| `k6/fixtures/auth.json` | Expected auth response schemas (token issuance, error envelope) |
| `k6/fixtures/accounts.json` | Expected account response schemas (list/single envelope, create/update requests) |
| `k6/fixtures/transactions.json` | Expected transaction response schemas (list/single envelope, create/update requests) |

### k6 Test Harness (`k6/harness.ts`)

Shared utility module providing:
- `getAuthToken()` — Bootstrap token issuance with caching
- `authenticatedHeaders()` — Bearer token header builder
- `checkListEnvelope()` — Validates Firefly-III list envelope structure + pagination
- `checkSingleEnvelope()` — Validates Firefly-III single envelope structure
- `checkResourceStructure()` — Validates resource object (type, id, attributes, links)
- `checkNoContent()` — Validates 204 responses (DELETE operations)
- `checkErrorEnvelope()` — Validates Firefly-III error format (message + errors)
- `checkUnauthorized()` — Validates 401 responses
- `checkPaginationConsistency()` — Validates pagination meta values

### k6 Test Scripts

| File | Tests |
|------|-------|
| `k6/auth.ts` | Bootstrap token issuance, authenticated request, unauthenticated rejection, token revocation, revoked token rejection, invalid bootstrap key |
| `k6/accounts.ts` | List (with pagination/type filter), get, create, update, delete, verify deletion |
| `k6/transactions.ts` | List (with pagination/type filter), get, create, update, delete, verify deletion, list by account |

### k6 Runner Script (`k6/run-all.sh`)

Shell script that orchestrates:
1. Health check wait (30s timeout)
2. Seed data load via Bun
3. Sequential execution of all k6 test scripts
4. JSON results output to `k6-results/`

### Infrastructure

| File | Change |
|------|--------|
| `docker/docker-compose.yml` | Added `k6` service (grafana/k6:latest) with testing profile, volume mounts for k6/ and scripts/seed/, k6_results volume |
| `.github/workflows/ci-phase2.yml` | CI Phase 2 workflow (k6 Compatibility Verification Suite): start stack, health check, seed data, run k6 suite, upload artifacts |
| `.env.example` | Added `APP_BASE_URL` and `K6_OUTPUT_DIR` variables |

## How to Run

### Local Development
```bash
# 1. Start the application stack
docker compose up -d

# 2. Load seed data
bun run scripts/seed/index.ts

# 3. Run the full k6 suite
./k6/run-all.sh

# Or run individual test files
k6 run k6/auth.ts
k6 run k6/accounts.ts
k6 run k6/transactions.ts
```

### Docker Compose (k6 profile)
```bash
# Start stack with testing profile
docker compose --profile testing up -d

# Run tests inside the k6 container
docker compose run --rm k6 run /scripts/k6/auth.ts
docker compose run --rm k6 run /scripts/k6/accounts.ts
docker compose run --rm k6 run /scripts/k6/transactions.ts
```

### CI (GitHub Actions)
The ci-phase2.yml workflow (CI Phase 2 — k6 Compatibility Verification Suite) runs automatically on every PR to main. Results are uploaded as artifacts.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    GitHub Actions CI                      │
│                                                          │
│  ┌─────────┐   ┌──────────┐   ┌──────────┐   ┌───────┐  │
│  │  Start  │──▶│  Health  │──▶│  Seed    │──▶│  k6   │  │
│  │  Stack  │   │  Check   │   │  Data    │   │ Tests │  │
│  └─────────┘   └──────────┘   └──────────┘   └───┬───┘  │
│                                                   │      │
│                                              ┌────▼────┐ │
│                                              │ Upload  │ │
│                                              │ Results │ │
│                                              └─────────┘ │
└──────────────────────────────────────────────────────────┘

Docker Compose Stack:
  ┌──────────┐    ┌──────────┐    ┌──────────┐
  │  Caddy   │───▶│  Utopia  │───▶│ Postgres │
  │  (80/443)│    │  API     │    │   (DB)   │
  └──────────┘    └────┬─────┘    └──────────┘
                       │
                  ┌────▼─────┐
                  │    k6    │
                  │ (testing)│
                  └──────────┘
```

## Compatibility Coverage

The suite validates Firefly-III v6.x response format for:
- **Auth**: Token issuance envelope, error format, 401 rejection
- **Accounts**: List envelope with pagination, single envelope, resource structure (type/id/attributes/links), CRUD operations
- **Transactions**: List envelope with pagination, single envelope, resource structure, CRUD operations, account-scoped listing

## NFR Compliance

| NFR | Implementation |
|-----|----------------|
| PRF-001 (< 2 min) | 1 VU, sequential execution, no throttling |
| PRF-002 (< 500ms) | k6 threshold: `http_req_duration: ["p(95)<500"]` |
| REL-001 (Idempotency) | Deterministic seed data, truncate+reseed before each run |
| REL-002 (Isolation) | `TRUNCATE ... CASCADE` before seed |
| REL-003 (DB Reset) | Seed script performs truncate + reseed |
| OBS-001 (Reporting) | JSON output with per-test pass/fail + timing |
| OBS-002 (Artifacts) | JSON reports uploaded as CI artifacts |
| SEC-001 (Seed Data) | Synthetic fixtures only, no real credentials |
| SEC-002 (Isolation) | k6 runs in Docker Compose network, no production access |
| MAI-001 (Versioning) | Programmatic TypeScript seed generation |
| MAI-002 (Language) | TypeScript throughout (Bun + k6) |
| MAI-003 (Extensibility) | Per-domain files with shared harness pattern |
| INF-001 (Docker) | k6 service added to Docker Compose |
| INF-002 (CI) | GitHub Actions workflow on every PR |

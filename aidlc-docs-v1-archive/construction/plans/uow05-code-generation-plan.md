# Code Generation Plan — UOW-05: Compatibility Verification Suite

## Prerequisites
- NFR Requirements: COMPLETED
- NFR Design: COMPLETED
- Infrastructure Design: SKIPPED (within existing Docker Compose topology)
- Functional Design: SKIPPED (test infrastructure, no business logic)
- Application is brownfield (Rust/Axum codebase already exists)
- Code location: workspace root (never aidlc-docs/)

## Unit Context
- **Unit**: Compatibility Verification Suite — k6-based automated test harness
- **Scope**: Test infrastructure only (no application source code changes)
- **Dependencies**: Existing Docker Compose stack, existing GitHub Actions CI
- **Stories**: No user stories (test infrastructure, no user-facing features)

## Part 1 - Planning Checklist
- [x] Analyze unit context and design artifacts
- [x] Create detailed code generation plan
- [x] Present plan for approval

## Part 2 - Generation Execution Checklist
- [x] Step 1: Create seed data generator (scripts/seed/)
- [x] Step 2: Create k6 test fixtures (k6/fixtures/)
- [x] Step 3: Create k6 test harness (k6/harness.ts)
- [x] Step 4: Create auth endpoint tests (k6/auth.ts)
- [x] Step 5: Create accounts endpoint tests (k6/accounts.ts)
- [x] Step 6: Create transactions endpoint tests (k6/transactions.ts)
- [x] Step 7: Create k6 runner script (k6/run-all.sh)
- [x] Step 8: Update Docker Compose with k6 service (docker/docker-compose.yml)
- [x] Step 9: Create GitHub Actions workflow for k6 (.github/workflows/compatibility-check.yml)
- [x] Step 10: Update .env.example with k6 environment variables
- [x] Step 11: Create code summary document
- [x] Step 12: Validate all artifacts

---

## Generation Steps (Detailed)

### Step 1: Create Seed Data Generator (scripts/seed/)
**Target files**:
- `scripts/seed/package.json` — dependencies (pg, dotenv)
- `scripts/seed/index.ts` — entry point: truncate + seed
- `scripts/seed/types.ts` — shared TypeScript interfaces
- `scripts/seed/accounts.ts` — account fixture definitions
- `scripts/seed/transactions.ts` — transaction fixture definitions

**Description**: Create a TypeScript seed data generator using Bun runtime. The script truncates all tables, then inserts deterministic test fixtures for accounts, transactions, and a test user. Uses the `pg` library for PostgreSQL connectivity.

---

### Step 2: Create k6 Test Fixtures (k6/fixtures/)
**Target files**:
- `k6/fixtures/auth.json` — expected auth response structures
- `k6/fixtures/accounts.json` — expected account response structures
- `k6/fixtures/transactions.json` — expected transaction response structures

**Description**: Create JSON fixture files with expected response schemas for strict mode comparison. Dynamic fields (timestamps, UUIDs) use wildcard placeholders.

---

### Step 3: Create k6 Test Harness (k6/harness.ts)
**Target files**:
- `k6/harness.ts` — shared test harness module

**Description**: Create shared utility module with base URL configuration, auth token retrieval, response validation helpers, and dynamic field exclusion logic for strict mode comparison.

---

### Step 4: Create Auth Endpoint Tests (k6/auth.ts)
**Target files**:
- `k6/auth.ts` — authentication endpoint tests

**Description**: Create k6 test script for auth endpoints: token issuance (POST /api/v1/bootstrap/tokens), token revocation, and unauthenticated request rejection. Uses shared harness and fixtures.

---

### Step 5: Create Accounts Endpoint Tests (k6/accounts.ts)
**Target files**:
- `k6/accounts.ts` — account endpoint tests

**Description**: Create k6 test script for account endpoints: list (GET /api/v1/accounts), get, create, update, delete. Validates responses against Firefly-III format using fixtures.

---

### Step 6: Create Transactions Endpoint Tests (k6/transactions.ts)
**Target files**:
- `k6/transactions.ts` — transaction endpoint tests

**Description**: Create k6 test script for transaction endpoints: list (GET /api/v1/transactions), get, create, update, delete, list by account. Validates responses against Firefly-III format.

---

### Step 7: Create k6 Runner Script (k6/run-all.sh)
**Target files**:
- `k6/run-all.sh` — shell script to run all k6 tests

**Description**: Create a shell script that orchestrates the full test suite execution: health check wait, seed data load, run all k6 tests, output results.

---

### Step 8: Update Docker Compose with k6 Service
**Target files**:
- `docker/docker-compose.yml` — add k6 service

**Description**: Add a k6 service to the existing Docker Compose configuration. The service mounts the k6/ and scripts/seed/ directories and depends on the app service health check.

---

### Step 9: Create GitHub Actions Workflow
**Target files**:
- `.github/workflows/compatibility-check.yml` — CI workflow for k6 tests

**Description**: Create a GitHub Actions workflow that triggers on every PR. Steps: start stack, wait for health, seed data, run k6 suite, upload JSON + HTML reports as artifacts.

---

### Step 10: Update .env.example
**Target files**:
- `.env.example` — add k6-related environment variables

**Description**: Add k6-specific environment variables: APP_BASE_URL, SEED_DATA_PATH, K6_OUTPUT_DIR.

---

### Step 11: Create Code Summary Document
**Target files**:
- `aidlc-docs/construction/compatibility-verification-suite/code/code-summary.md`

**Description**: Create a markdown summary of all generated files, their purposes, and how to run the test suite.

---

### Step 12: Validate All Artifacts
**Target**: Verify all files are in correct locations, no aidlc-docs/ pollution, Docker Compose syntax valid, YAML workflow valid.

---

[Answer]: Ready to proceed with Code Generation for UOW-05 Compatibility Verification Suite.

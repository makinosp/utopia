# NFR Requirements — UOW-05: Compatibility Verification Suite

## Scope
This document defines non-functional requirements for the Compatibility Verification Suite — a k6-based automated test harness that validates Utopia's API responses against Firefly-III compatibility requirements.

## Baseline Profile
- Deployment profile: self-hosted, Docker Compose stack (k6 container added).
- Availability posture: test infrastructure (no SLA commitment for test execution).
- Compatibility target: Firefly-III v6.x (latest stable).
- Endpoint scope: currently implemented endpoints only (Auth, Accounts, Transactions).

## Performance Requirements

### PRF-001 Full Suite Execution Time
- Target: < 2 minutes for the complete test suite against a local Docker Compose stack.
- Rationale: 1 VU shared iteration mode with sequential scenario execution; conservative CI budget.

### PRF-002 Per-Test Case Latency
- Target: Each individual HTTP request completes within 500ms (excluding network overhead).
- Rationale: Compatibility tests validate response correctness, not load capacity.

## Reliability Requirements

### REL-001 Idempotency (NFR-002)
- Tests MUST be idempotent — repeated execution against the same seed data produces identical pass/fail outcomes.
- Rationale: Deterministic test results are required for CI gate reliability.

### REL-002 Isolation (NFR-003)
- Each test run MUST start from a known database state.
- Mechanism: Truncate all tables and re-insert seed data before each test execution.
- Rationale: Prevents cross-test contamination from shared database state.

### REL-003 Database Reset Strategy
- Approach: `TRUNCATE ... CASCADE` followed by seed data re-insertion.
- Rationale: Faster than drop + recreate; avoids DDL overhead while ensuring clean state.

## Observability Requirements

### OBS-001 Test Reporting (NFR-004)
- k6 output MUST include full detail: pass/fail per individual test case with timing breakdown.
- On failure: include request/response diffs for strict mode comparison.
- Rationale: Enables rapid diagnosis of compatibility regressions.

### OBS-002 Output Artifacts
- Retain both JSON summary report (machine-parseable for CI gate) and HTML report (human-readable for debugging).
- Rationale: JSON enables automated CI pass/fail determination; HTML aids developer investigation.

## Security Requirements

### SEC-001 Seed Data Handling
- Seed data MUST NOT contain real user credentials or sensitive information.
- Seed data uses synthetic, deterministic fixtures only.
- Rationale: Test data is loaded into non-production environments but should follow security best practices.

### SEC-002 k6 Container Isolation
- k6 container MUST run in an isolated network namespace within Docker Compose.
- k6 container MUST NOT have access to production databases or secrets.
- Rationale: Prevents accidental production impact from test execution.

## Maintainability Requirements

### MAI-001 Seed Data Versioning
- Seed data MUST be generated programmatically using TypeScript with Bun runtime.
- Source location: `scripts/seed/index.ts`.
- Execution: `bun run scripts/seed/index.ts`.
- Rationale: Programmatic generation avoids drift when schema changes; TypeScript enables type sharing with k6 test scripts.

### MAI-002 Language and Runtime
- Seed generation scripts: TypeScript with Bun runtime.
- k6 test scripts: TypeScript (executed by k6's built-in TypeScript support).
- Rationale: Unified TypeScript codebase for all test infrastructure; Bun provides fast execution without compilation step.

### MAI-003 Extensibility
- Test suite MUST be designed for extension — adding tests for new endpoints follows the established pattern without modifying existing test logic.
- Rationale: Future UOWs will implement additional endpoints; the suite must accommodate growth.

## Infrastructure Requirements

### INF-001 Docker Compose Integration
- A k6 service MUST be added to the Docker Compose configuration.
- The k6 service depends on the application stack being fully initialized (health check passing).
- Rationale: Ensures tests run against the same application stack used in development and CI.

### INF-002 CI Integration (FR-007)
- The k6 test suite MUST execute on every pull request via GitHub Actions.
- Full suite runs on every PR (not smoke-only).
- Rationale: Ensures compatibility is validated against all changes.

## Execution Configuration

### Execution Mode
- k6 execution mode: Shared iteration (single VU processes all scenarios sequentially).
- Virtual Users: 1 VU.
- Rationale: Deterministic execution order; avoids shared DB contention; simplifies debugging.

### CI Execution Command
```bash
# Start application stack
docker compose up -d

# Wait for health checks
./scripts/wait-for-healthy.sh

# Reset database and load seed data
bun run scripts/seed/index.ts

# Run k6 compatibility suite
docker compose run k6 run /scripts/k6/all.js

# Tear down
docker compose down
```

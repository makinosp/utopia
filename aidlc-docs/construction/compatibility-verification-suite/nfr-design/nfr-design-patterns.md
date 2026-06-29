# NFR Design Patterns — UOW-05: Compatibility Verification Suite

## Overview
This document defines the non-functional design patterns applied to the Compatibility Verification Suite, based on approved NFR requirements and design decisions.

## Resilience Patterns

### RLP-001 Fail-Fast on Test Failure
- When a test fails due to any issue (transient or permanent), the suite marks it as failed immediately and continues to the next test.
- No retry logic is applied to individual test assertions.
- Rationale: Compatibility tests validate deterministic API behavior. Retrying masks real compatibility issues. In a local Docker Compose environment, transient failures indicate infrastructure problems, not compatibility problems.

### RLP-002 Pre-Test Health Check
- Before executing the k6 suite, perform a simple HTTP health check against the application's `/health` endpoint.
- Wait for HTTP 200 OK response before proceeding with tests.
- Timeout: 30 seconds (fail the suite if the stack does not become ready).
- Rationale: Prevents wasting CI time on tests that would all fail due to an unready stack. A simple health check is sufficient — deeper issues surface as individual test failures.

## Performance Patterns

### PRF-001 No Request Throttling
- Requests are fired as fast as the single VU can execute them.
- No artificial delays between requests.
- Rationale: The suite uses 1 VU against a local Docker Compose stack. The local stack handles normal application traffic without issue. Throttling would increase execution time without benefit.

## Security Patterns

### SEC-001 Shared Docker Network
- The k6 container shares the same Docker Compose network as the application and database.
- No network isolation or explicit firewall rules.
- Rationale: This is a test-only stack with no production data. All containers are ephemeral (destroyed after test execution). Network isolation adds configuration complexity without security benefit.

## Observability Patterns

### OBS-001 Custom Tags with k6 Default Metrics
- Each k6 test case is tagged with its endpoint group (e.g., `endpoint:auth`, `endpoint:accounts`, `endpoint:transactions`).
- k6's built-in metrics are retained (http_req_duration, http_req_failed, http_reqs, etc.).
- Tags enable filtered analysis in the JSON report.
- Rationale: Custom tags provide domain-specific grouping without sacrificing k6's rich built-in metrics. This enables both high-level summary and detailed investigation.

### OBS-002 Dual Output Artifacts
- JSON summary report: machine-parseable, used for CI gate evaluation.
- HTML report: human-readable, used for manual investigation on failure.
- Rationale: JSON enables automated pass/fail determination in CI. HTML provides visual diff and timing information for developers.

## Scalability Patterns

### SCL-001 Per-Domain Test Files with Shared Harness
- Test files are organized per domain: `k6/auth.ts`, `k6/accounts.ts`, `k6/transactions.ts`.
- Shared fixture modules: `k6/fixtures/auth.ts`, `k6/fixtures/accounts.ts`, `k6/fixtures/transactions.ts`.
- Common test harness: `k6/harness.ts` provides shared utilities (base URL configuration, auth token retrieval, response validation helpers).
- Rationale: Per-domain files enable independent development and review. Shared fixtures ensure consistent test data setup. The harness eliminates duplication of common logic. New domains can be added by creating a new file following the established pattern.

## Reliability Patterns

### REL-001 Deterministic Execution Order
- Tests execute in a fixed, sequential order within each domain file.
- Domain files are executed in a defined order: auth → accounts → transactions.
- Rationale: Deterministic ordering makes failures reproducible and simplifies debugging. Shared database state requires careful ordering to prevent cross-test contamination.

### REL-002 Database Reset Before Each Run
- Before each test execution, all tables are truncated and seed data is re-inserted.
- Mechanism: `bun run scripts/seed/index.ts` performs truncate + seed.
- Rationale: Ensures each test run starts from a known, clean database state (NFR-003 Isolation).

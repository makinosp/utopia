# UOW-05: Compatibility Verification Suite — Requirements

## Intent Analysis Summary
- **User request**: Add a new Unit of Work for a k6-based automated compatibility verification suite that validates API behavior against Firefly-III specifications.
- **Request type**: Enhancement (test infrastructure).
- **Scope estimate**: Single component (test/k6 scripts, Docker Compose integration, CI workflow updates).
- **Complexity estimate**: Moderate (k6 scripting, OpenAPI schema validation, seed data management, CI integration).

## Overview
UOW-05 is a **Compatibility Verification Suite** — an automated test harness using [k6](https://k6.io/) that validates Utopia's API responses against Firefly-III compatibility requirements. The suite verifies that existing client applications (e.g., Waterfly-III) can connect to Utopia without modification for supported endpoints.

## Target Compatibility
- **Firefly-III version**: v6.x (latest stable).
- **Compatibility scope**: Currently implemented endpoints only (Accounts, Transactions, Auth/Core).
- **Verification approach**: k6 test scripts validate API responses against expected Firefly-III response shapes and content.

## Functional Requirements

### FR-001: k6 Test Suite
Implement a k6-based test suite that sends HTTP requests to Utopia API endpoints and validates responses.

### FR-002: OpenAPI Schema Validation
Validate API responses against the Firefly-III OpenAPI specification for structural correctness (field presence, data types, required fields).

### FR-003: Strict Mode Response Matching
Support strict mode validation — full JSON response matching (excluding dynamic fields such as timestamps, UUIDs, and request IDs).

### FR-004: Sample Response Fixtures
Provide hand-defined sample response JSON files based on the Firefly-III OpenAPI spec for strict mode comparison.

### FR-005: Seed Data Management
Prepare seed SQL data in advance and load it into the test database before test execution, ensuring deterministic test outcomes.

### FR-006: Docker Compose Integration
Add a k6 container to the Docker Compose environment so that tests run against the same application stack used in development and CI.

### FR-007: CI Integration
Integrate the k6 test suite into GitHub Actions from the beginning, automatically running on PR creation.

### FR-008: Endpoint Coverage
Cover the following endpoint groups:
- **Authentication**: Token issuance, token revocation, unauthenticated request rejection.
- **Accounts**: List, get, create, update, delete.
- **Transactions**: List, get, create, update, delete, list by account.

## Non-Functional Requirements

### NFR-001: Execution Performance
The k6 test suite should complete within a reasonable time frame (target: < 2 minutes for the full suite against a local stack).

### NFR-002: Idempotency
Tests must be idempotent — repeated execution against the same seed data produces the same results.

### NFR-003: Isolation
Each test run must start from a known database state (seed data restored before execution).

### NFR-004: Observability
k6 test output must clearly report pass/fail per endpoint, with diff details on failure for strict mode comparisons.

## Key Decisions
1. **Golden response data**: Manually defined from Firefly-III OpenAPI spec (Option B) — combines OpenAPI schema validation with sample response JSON files.
2. **Pass/fail criteria**: Strict mode — full JSON response match excluding dynamic fields (Option A).
3. **Execution environment**: k6 container in Docker Compose; run in CI via GitHub Actions (Option A).
4. **Test data management**: Pre-prepared seed SQL data loaded before tests (Option B).
5. **CI integration**: From the beginning, automatic on PR creation (Option A).

## Constraints and Assumptions
1. UOW-05 does not modify any application source code — it is test infrastructure only.
2. The suite extends automatically as new endpoints are implemented in future UOWs.
3. Seed data must remain synchronized with the database migration schema.
4. Dynamic fields (timestamps, UUIDs, request IDs) are excluded from strict comparison using k6 response transformation.

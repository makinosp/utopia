# NFR Design Plan — UOW-05: Compatibility Verification Suite

## Prerequisites
- NFR Requirements: COMPLETE
- NFR requirements artifacts: `aidlc-docs/construction/compatibility-verification-suite/nfr-requirements/`
- Execution plan: NFR Design follows NFR Requirements in the construction loop

## Part 1 - Planning Checklist
- [x] Analyze NFR requirements artifacts
- [x] Generate context-appropriate questions for NFR design refinement
- [x] Collect and analyze answers
- [x] Generate nfr-design-patterns.md and logical-components.md
- [x] Move stage to awaiting approval

## Part 2 - Generation Execution Checklist
- [x] Read approved plan and identify first uncompleted generation step
- [x] Generate nfr-design-patterns.md
- [x] Mark step complete
- [x] Generate logical-components.md
- [x] Mark step complete
- [x] Validate NFR design artifacts
- [ ] Mark NFR Design stage complete (awaiting user approval)

---

## NFR Design Questions

Please fill all `[Answer]:` fields.

### Question 1: Resilience Pattern — Test Failure Handling

When a test fails due to a transient issue (e.g., temporary DB connection timeout), how should the suite behave?

A) **Fail fast** — mark the test as failed immediately and continue to the next test
B) **Retry once** — retry the failed request once before marking as failed
C) **Retry with backoff** — retry up to 3 times with exponential backoff before marking as failed
D) Other (please specify after [Answer]:)

[Answer]: A

---

### Question 2: Resilience Pattern — Health Check Before Tests

Before running the k6 suite, should there be a health check to verify the application stack is ready?

A) **No health check** — assume the stack is ready; let connection failures surface as test failures
B) **Simple HTTP health check** — ping `/health` endpoint and wait for 200 OK before proceeding
C) **Full readiness check** — verify DB connectivity, API responsiveness, and seed data presence before tests
D) Other (please specify after [Answer]:)

[Answer]: B

---

### Question 3: Performance Pattern — Request Throttling

Should the k6 suite include deliberate pacing/throttling between requests to avoid overwhelming the local stack?

A) **No throttling** — fire requests as fast as possible (local stack can handle it)
B) **Fixed delay** — add a small fixed delay (e.g., 100ms) between requests
C) **Configurable rate** — allow rate configuration via environment variable
D) Other (please specify after [Answer]:)

[Answer]: A

---

### Question 4: Security Pattern — k6 Container Network

How should the k6 container be networked within Docker Compose?

A) **Shared network** — k6 shares the same network as the application and database (simplest)
B) **Isolated network with app access** — k6 can reach the app but not the database directly
C) **Fully isolated** — k6 can only reach the app via exposed ports; no DB access at all
D) Other (please specify after [Answer]:)

[Answer]: A

---

### Question 5: Observability Pattern — Structured Test Output

How should individual test results be structured in the k6 output for CI parsing?

A) **k6 default metrics** — rely on built-in k6 metrics (http_req_duration, http_req_failed, etc.)
B) **Custom tags + metrics** — add custom tags per endpoint group (e.g., `endpoint:auth`, `endpoint:accounts`) plus k6 defaults
C) **Custom JSON events** — emit structured JSON events per test case for external parsing
D) Other (please specify after [Answer]:)

[Answer]: B

---

### Question 6: Scalability Pattern — Test Suite Modularity

How should the test files be organized as the suite grows?

A) **Single file** — one `all.ts` file containing all endpoint tests (simplest)
B) **Per-domain files** — separate files per domain (`auth.ts`, `accounts.ts`, `transactions.ts`) with a shared runner
C) **Per-domain + shared fixtures** — separate files with shared fixture modules and a common test harness
D) Other (please specify after [Answer]:)

[Answer]: C

---

### Question 7: Logical Components — Test Data Fixtures

Where should the expected response fixtures (for strict mode comparison) be stored?

A) **Inline in test files** — define expected JSON directly in each test file
B) **Shared fixture directory** — `k6/fixtures/` with per-domain JSON files
C) **Generated from OpenAPI** — auto-generate fixtures from the Firefly-III OpenAPI spec at build time
D) Other (please specify after [Answer]:)

[Answer]: B

---

### Question 8: Logical Components — Dynamic Field Handling

For strict mode comparison (FR-003), how should dynamic fields (timestamps, UUIDs, request IDs) be excluded?

A) **Strip before comparison** — remove dynamic fields from the response before comparing
B) **Wildcard matching** — use a schema-based matcher that ignores dynamic fields
C) **JSONPath extraction** — compare only specific non-dynamic fields via JSONPath
D) Other (please specify after [Answer]:)

[Answer]: B

---

[Answer]: Ready to proceed to NFR Design stage with the above plan.

# NFR Requirements Plan - Accounts Module (UOW-02)

## Objective
Define non-functional requirements and technology decisions for the Accounts Module (UOW-02) with emphasis on security enforcement (ownership validation), performance of dynamic listings, and data integrity.

## Context Inputs
- Unit: UOW-02 Accounts Module
- Functional design plan:
  - aidlc-docs/construction/plans/accounts-functional-design-plan.md
  - Key scope additions: Full CRUD (5 endpoints), 50-attribute response schema, balance calculation (initial_balance + transaction sums), soft delete hybrid policy, context-derived currency defaults
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## NFR Planning Checklist
- [x] Analyze risk profile of data leakage between accounts of different users — COMPLETED
- [x] Define performance targets for account listing and read/write operations — COMPLETED
- [x] Establish latency limits for dynamic balance queries/calculations — COMPLETED
- [x] Define input validation strictness guidelines (account names, attributes, rates) — COMPLETED
- [x] Decide auditing/logging requirements for critical resource CRUD modifications — COMPLETED
- [x] Generate NFR requirements artifacts — COMPLETED
- [x] Request approval to proceed to NFR Design

## Planning Questions

Please fill all `[Answer]:` fields with your design preferences.

### Question 1: Performance / Latency SLA

What is the target p95 latency for listing accounts (with pagination/filters) under baseline self-hosted database sizes (e.g. 100 accounts per user)?

A) <= 100ms
B) <= 50ms
C) <= 20ms
D) <= 10ms
X) Other (please describe)

[Answer]: B

**Design Context**: The functional design adds dynamic balance calculation (`initial_balance + SUM(transactions)`) for `date`-filtered queries and soft-delete filtering (`WHERE deleted_at IS NULL`). Both operations must stay within the 50ms p95 SLA. If dynamic balance calculation for historical dates exceeds this threshold, a materialized balance snapshot or caching layer should be considered during NFR Design.

### Question 2: Audit Logging for Modifications

To what level should account modifications (Creates, Updates, Deletions) be logged for system auditing?

A) Minimal: Log error events only
B) standard: Log event type, timestamp, user ID, and targeted Account ID
C) Extended: Log full target change payloads (excluding secrets) for tracing and change auditing
D) Other (please describe)

[Answer]: B

**Design Context**: The functional design specifies soft delete (hidden from API) and hard delete (when no transactions exist). Audit logs must distinguish between these two deletion types.

### Question 3: Concurrency Control on Account Writes

To prevent concurrent modifications of the same account records or conflicting entries (such as concurrent requests attempting to create an account with the same name), what isolation or lock level should be requested?

A) Standard SQL `READ COMMITTED` isolation (standard Postgres default, risk of race condition if handled carelessly)
B) Pessimistic Locking (`SELECT FOR UPDATE`) on critical reads
C) Optimistic Locking with a tracking version column (`version int`)
D) Other (please describe)

[Answer]: A

**Design Context**: The functional design introduces `initial_balance` storage and automatic "Opening balance" transaction creation on account creation with `opening_balance`. These must be wrapped in a single database transaction to ensure atomicity. `READ COMMITTED` is acceptable as long as the creation flow uses a single transaction.

### Question 4: Input Validation Strictness

What is the maximum allowed length for Account Name strings to mitigate overflow/DoS risks?

A) 255 characters (standard database field size limit)
B) 100 characters (practical frontend constraint)
C) Custom limit (please specify)

[Answer]: A

**Design Context**: The functional design also adds IBAN validation (format check on creation/update). IBAN validation must not introduce measurable latency (>5ms). Consider a lightweight regex-based format check as an initial implementation, with full modulus-97 validation planned for Phase 2.

---

## Derived NFR Requirements

Based on the planning answers and the expanded functional design scope, the following concrete NFR requirements are defined:

| ID | Requirement | Category | Verification |
|---|---|---|---|
| NFR-ACCT-01 | Account listing (with pagination + type filter) must complete within 50ms p95 for ≤100 accounts per user | Performance | Benchmark test with 100 seeded accounts |
| NFR-ACCT-02 | Dynamic balance calculation with `date` parameter must complete within 100ms p95 for ≤1000 transactions per account | Performance | Benchmark with transaction history |
| NFR-ACCT-03 | Account creation (including opening balance transaction) must be atomic within a single DB transaction | Data Integrity | Integration test with forced failure mid-creation |
| NFR-ACCT-04 | All account modifications (create/update/delete) must be logged with event type, timestamp, user ID, and account ID | Audit | Log inspection test |
| NFR-ACCT-05 | Soft-deleted accounts must be excluded from all API responses by default | Security | Query inspection and integration test |
| NFR-ACCT-06 | Cross-user account access must return 404 (never 403) | Security | Integration test |
| NFR-ACCT-07 | Account name must not exceed 255 characters | Input Validation | Boundary test |
| NFR-ACCT-08 | IBAN format validation must complete within 5ms | Performance | Micro-benchmark |
| NFR-ACCT-09 | Duplicate account name (same user) must be rejected with 422 and appropriate error message | Data Integrity | Integration test |

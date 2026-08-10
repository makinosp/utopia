# NFR Requirements Plan - Transactions Module (UOW-03)

## Objective
Define non-functional requirements and technology decisions for the Transactions Module (UOW-03) with emphasis on financial data integrity (atomic balance updates), security (ownership enforcement via account cross-validation), and listing performance.

## Context Inputs
- Unit: UOW-03 Transactions Module
- Functional design plan:
  - aidlc-docs/construction/plans/transactions-functional-design-plan.md
  - Key scope additions: 6 endpoints (list, get, create, update, delete transactions + list account transactions), atomic balance updates on create/update/delete, source/destination account validation, transaction type rules (withdrawal/deposit/transfer)
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## NFR Planning Checklist
- [x] Analyze risk profile of financial data integrity (balance calculation errors)
- [x] Define performance targets for transaction listing with date-range filters
- [x] Establish latency limits for atomic balance updates within DB transactions
- [x] Define concurrency control requirements for balance-affecting operations
- [x] Decide auditing/logging requirements for transaction modifications
- [x] Generate NFR requirements artifacts
- [x] Request approval to proceed to NFR Design

## Planning Questions

Please fill all `[Answer]:` fields with your design preferences.

### Question 1: Performance / Latency SLA

What is the target p95 latency for listing transactions (with pagination and optional date-range/type filters) under baseline self-hosted database sizes (e.g. 10,000 transactions per user)?

A) <= 200ms
B) <= 100ms
C) <= 50ms
D) <= 20ms
X) Other (please describe)

[Answer]: B

**Design Context**: The functional design adds composite indexes on `(user_id, date DESC)` and per-account `(user_id, source_id/destination_id, date DESC)`. The p95 latency must account for the dynamic SQL query building needed for optional filters (start/end dates, type). With proper indexes, 100ms is achievable for 10k records.

### Question 2: Atomic Balance Integrity

What isolation mechanism should be used for balance-affecting transaction operations (create/update/delete) to prevent race conditions?

A) Standard SQL `READ COMMITTED` (default) — sufficient because all balance updates and journal writes happen atomically in a single DB transaction
B) `REPEATABLE READ` isolation for write operations
C) `SERIALIZABLE` isolation for all balance-affecting flows
D) Pessimistic locking (`SELECT FOR UPDATE`) on affected account rows within the transaction

[Answer]: D

**Design Context**: The functional design specifies that balance updates (`current_balance += delta`) happen within the same DB transaction as the journal CRUD. To prevent race conditions when two concurrent transactions affect the same account, `SELECT FOR UPDATE` on the affected account rows inside the transaction provides the strongest guarantee without the overhead of `SERIALIZABLE` isolation.

### Question 3: Audit Logging for Modifications

To what level should transaction modifications (Creates, Updates, Deletions) be logged for system auditing?

A) Minimal: Log error events only
B) Standard: Log event type, timestamp, user ID, and transaction ID
C) Extended: Log full transaction payload including source/destination account IDs and amount changes
D) Other (please describe)

[Answer]: B

**Design Context**: Financial transactions represent critical audit events. At minimum, the audit log must record the operation type (create/update/delete), the affected transaction ID, the user who performed it, and the timestamp. Extended payload logging (including amounts) is deferred to Phase 2 since the current audit infrastructure may not support structured payload capture.

### Question 4: Input Validation Strictness

What is the maximum allowed length for transaction description strings to mitigate overflow/DoS risks?

A) 255 characters (standard database field size limit)
B) 100 characters (practical frontend constraint)
C) Custom limit (please specify)

[Answer]: A

### Question 5: Concurrency Conflict Handling

When a balance-affecting update or delete operation fails mid-transaction (e.g., due to a concurrent modification), what should the system do?

A) Return a 409 Conflict error with a descriptive message
B) Retry the transaction automatically (up to 3 attempts)
C) Return 500 Internal Server Error (standard persistence error)
D) Queue the operation for later retry via a background worker

[Answer]: A

**Design Context**: While `SELECT FOR UPDATE` (Question 2) prevents most race conditions, deadlocks or serialization failures can still occur. The API should return 409 Conflict for such cases to allow clients to retry with fresh state, rather than silently failing with a generic 500.

### Question 6: Property-Based Testing Scope

How strict should property-based testing be for this unit?

A) Apply only to decimal serialization round-trips (shared with UOW-01)
B) Apply to serialization + balance calculation invariants (`balance_impacts` and `reverse_balance_impacts` functions)
C) Apply to serialization + balance invariants + transaction type validation rules
D) Skip PBT for this unit; use example-based tests only

[Answer]: B

**Design Context**: The `balance_impacts` and `reverse_balance_impacts` pure functions are ideal candidates for property-based testing. The invariants "reverse(reverse(x)) == x" and "sum of balance deltas matches transaction semantics" should be verified with generated positive/negative decimal amounts. Transaction type validation rules are straightforward enough for example-based tests.

---

## Derived NFR Requirements

Based on the planning answers and the functional design scope, the following concrete NFR requirements are defined:

| ID | Requirement | Category | Verification |
|---|---|---|---|
| NFR-TXN-01 | Transaction listing (with pagination + optional date/type filters) must complete within 100ms p95 for ≤10,000 transactions per user | Performance | Benchmark test with 10k seeded transactions |
| NFR-TXN-02 | Balance-affecting operations (create/update/delete) must use `SELECT FOR UPDATE` on affected account rows within the DB transaction | Data Integrity | Code review and integration test with concurrent access |
| NFR-TXN-03 | Balance updates must be atomic: if the transaction journal insert succeeds, the balance update must also succeed; if either fails, both must roll back | Data Integrity | Integration test with forced failure mid-operation |
| NFR-TXN-04 | All transaction modifications (create/update/delete) must be logged with event type, timestamp, user ID, and transaction ID | Audit | Log inspection test |
| NFR-TXN-05 | Cross-user access to transactions must return 404 (never 403) | Security | Integration test |
| NFR-TXN-06 | Transaction description must not exceed 255 characters | Input Validation | Boundary test |
| NFR-TXN-07 | Source/destination account existence and ownership must be validated before any balance-affecting write | Security | Integration test |
| NFR-TXN-08 | Amount must be a positive decimal value; zero or negative amounts must be rejected with 422 | Input Validation | Boundary test |
| NFR-TXN-09 | `balance_impacts` and `reverse_balance_impacts` functions must satisfy `reverse(reverse(x)) == x` for all valid transaction types and positive amounts | Property-Based Testing | Property test with generated data |
| NFR-TXN-10 | Concurrent balance conflicts must return 409 Conflict with descriptive error message | Concurrency | Integration test with concurrent requests |

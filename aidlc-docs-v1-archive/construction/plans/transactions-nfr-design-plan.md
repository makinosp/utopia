# NFR Design Plan - Transactions Module (UOW-03)

## Objective
Translate approved NFR requirements into concrete non-functional design patterns and logical components for the Transactions Module.

## Context Inputs
- Unit: UOW-03 Transactions Module
- Source artifacts:
  - `aidlc-docs/construction/plans/transactions-functional-design-plan.md`
  - `aidlc-docs/construction/plans/transactions-nfr-requirements-plan.md`
  - `aidlc-docs/construction/core-foundation/nfr-design/nfr-design-patterns.md` (reuse applicable patterns)
  - `aidlc-docs/construction/core-foundation/nfr-design/logical-components.md` (reuse applicable components)
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## NFR Design Checklist
- [x] Define concurrency control pattern for balance-affecting operations — COMPLETED
- [x] Define atomicity pattern for balance updates within DB transactions — COMPLETED
- [x] Define performance patterns for date-range filtered listings — COMPLETED
- [x] Define security patterns for cross-account validation — COMPLETED
- [x] Define observability patterns (logging, metrics for transaction operations) — COMPLETED
- [x] Define logical components and integration boundaries — COMPLETED
- [x] Validate design consistency against approved NFR requirements — COMPLETED
- [x] Request approval to proceed to Infrastructure Design

## Design Decisions

### Decision 1: Concurrency Control for Balance Updates

How should concurrent balance-affecting operations on the same account be prevented from causing race conditions?

A) Standard `READ COMMITTED` isolation — risk of lost updates under concurrent writes
B) `SELECT FOR UPDATE` on affected account rows at the start of each write transaction
C) Optimistic locking with a version column on accounts
D) Application-level mutex per account ID

**[Answer]**: B — `SELECT FOR UPDATE` on affected account rows at the start of each write transaction. When a transaction is created, the service layer first selects the source/destination account rows with `FOR UPDATE` to lock them, then proceeds with the journal insert and balance update.

Rationale:
- Meets NFR-TXN-02 (concurrency control) with proven PostgreSQL pattern.
- Avoids the overhead and retry complexity of optimistic locking.
- No external infrastructure needed (unlike application-level mutex).
- The accounts are already being validated for existence/ownership in the same transaction, so `SELECT FOR UPDATE` can be combined with the existence check.

**Patterns affected**: Add a `lock_accounts_for_update` step to `TransactionWriteRepository` that accepts a list of account IDs and returns locked account rows.

### Decision 2: Balance Update Atomicity

How should the atomicity guarantee (NFR-TXN-03) be implemented?

A) Application-level dual-write: insert journal, then update balance — both in the same DB transaction
B) PostgreSQL trigger on transaction_journals table that automatically updates account balances
C) CTE (Common Table Expression) that inserts the journal and updates balances in a single SQL statement
D) Two-phase commit across separate services

**[Answer]**: A — Application-level dual-write within the same DB transaction. The `TransactionService` orchestrates the sequence: (1) lock accounts via `SELECT FOR UPDATE`, (2) insert journal record, (3) update affected account balances, (4) commit. If any step fails, the transaction rolls back entirely.

Rationale:
- Matches the existing pattern used in UOW-02 Accounts Module for account creation.
- Keeps balance logic explicit and testable in the service layer (not hidden in triggers).
- The `TransactionWriteRepository::update_account_balances` method already implements the balance delta updates.
- Easier to debug and maintain than database triggers.

### Decision 3: Account Name Resolution for Transaction Views

How should source/destination account names be resolved in transaction listing responses?

A) SQL JOIN in the repository query
B) Separate queries after fetching transaction records (N+1 or batched)
C) Deferred resolution: return only IDs, let the client resolve names
D) Materialized view combining transaction + account data

**[Answer]**: B — Separate batched queries after fetching transaction records. The service layer collects unique account IDs from the fetched transactions, queries account names in a single `SELECT ... WHERE id = ANY($1)` query, and maps them back in memory.

Rationale:
- Avoids complex JOIN logic in the dynamic-filter listing query (which already has dynamic SQL construction for optional date/type filters).
- For a typical page of 50 transactions, at most 100 unique account IDs need resolution — a single batched query is efficient.
- Keeps repository methods composable and testable.

### Decision 4: Transaction Deletion Strategy

Should deleted transactions be soft-deleted or hard-deleted?

A) Hard delete — permanently remove the journal record
B) Soft delete with `deleted_at` timestamp — hide from API but preserve data
C) Soft delete with data retention period, then archival
D) Hybrid: hard delete by default, soft delete configurable

**[Answer]**: A — Hard delete. Transaction journal records are permanently removed when the DELETE endpoint is called.

Rationale:
- Firefly-III uses hard delete for transactions (unlike accounts which use soft delete).
- Balance reversal logic is already implemented in the service layer.
- Simplified query logic without `WHERE deleted_at IS NULL` filters on every listing.
- Data integrity is maintained because the balance reversal happens atomically in the same transaction before the delete.

### Decision 5: Account Validation Caching

Should account existence/ownership checks (source/destination validation) be cached to reduce database round-trips?

A) No cache — always validate against the database in the write transaction
B) Short TTL in-memory cache for account existence (separate from the write transaction)
C) Always validate in the write transaction and combine with `SELECT FOR UPDATE`

**[Answer]**: C — Always validate in the write transaction and combine with `SELECT FOR UPDATE`. The validation check is merged with the concurrency lock step, so there's no additional round-trip. The `SELECT ... FOR UPDATE` on the account rows serves both as existence verification and as a lock.

Rationale:
- Combines NFR-TXN-07 (ownership validation) with NFR-TXN-02 (concurrency control).
- No additional latency for account validation.
- Avoids cache invalidation complexity.

### Decision 6: Audit Logging Implementation

How should transaction CRUD modifications be logged (NFR-TXN-04)?

A) Structured `tracing::info!` log events with event type, user_id, transaction_id, and timestamp
B) Dedicated `audit_log` database table
C) Core Foundation's existing Audit Logger component, extended with transaction-specific event types
D) Defer audit logging for transactions to Phase 2

**[Answer]**: C — Extend Core Foundation's Audit Logger (defined in `logical-components.md`). Transaction operations emit structured audit events through the same audit logger component, with new event types: `transaction_created`, `transaction_updated`, `transaction_deleted`.

Rationale:
- Reuses existing infrastructure (Pattern SEC-03 from Core Foundation).
- Consistent security event envelope across all modules.
- Same approach as UOW-02 Accounts Module (Decision 6).

### Decision 7: Conflict Error Response Format

How should concurrent modification conflicts (NFR-TXN-10) be communicated to API clients?

A) Standard 422 validation error with a descriptive message
B) 409 Conflict with Firefly-compatible error envelope
C) 500 Internal Server Error (generic persistence error)
D) 503 Service Unavailable (retry later)

**[Answer]**: B — 409 Conflict with Firefly-compatible error envelope. The response follows the same `FireflyErrorResponse` format used by all other API errors, with `message: "Conflict"` and `errors` containing descriptive text about the concurrent modification.

Rationale:
- 409 is the semantically correct HTTP status for concurrent modification conflicts.
- Consistent with the Firefly-III error contract format used throughout Utopia.
- Allows clients to distinguish conflict errors from validation errors (422) and server errors (500).

---

## Derived Constraints

| Constraint | Source Decision | Implication |
|---|---|---|
| `SELECT FOR UPDATE` must be used on affected account rows | Decision 1 | Service layer must execute a `SELECT ... WHERE id = $1 FOR UPDATE` before balance updates; requires a new repository method |
| Balance updates are application-level dual-writes in a DB transaction | Decision 2 | TransactionService orchestrates the exact sequence of lock → insert → update → commit/rollback |
| Account names resolved via batched separate queries | Decision 3 | TransactionReadRepository needs a batch account name lookup method |
| Transaction records are hard-deleted | Decision 4 | No `deleted_at` column needed; balance reversal before delete is mandatory |
| Account validation merged with SELECT FOR UPDATE | Decision 5 | Single DB round-trip for validation + locking |
| Audit via Core Foundation Audit Logger | Decision 6 | Requires Audit Logger interface to be extended with transaction event types |
| 409 Conflict for concurrent modification errors | Decision 7 | `map_domain_error` must be extended to handle conflict-type errors |

## Approval

All planning questions have been answered. The NFR Design patterns and logical components decisions are documented above.

**Total artifacts delivered**: 1 document, 7 design decisions.

Next stage: **Infrastructure Design** — define database schema, index strategy, and deployment configuration for the Transactions Module.

[Answer]: Awaiting user approval to proceed to Infrastructure Design.

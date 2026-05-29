# NFR Design Plan - Accounts Module (UOW-02)

## Objective
Translate approved NFR requirements into concrete non-functional design patterns and logical components for the Accounts Module.

## Context Inputs
- Unit: UOW-02 Accounts Module
- Source artifacts:
  - `aidlc-docs/construction/plans/accounts-functional-design-plan.md`
  - `aidlc-docs/construction/plans/accounts-nfr-requirements-plan.md`
  - `aidlc-docs/construction/core-foundation/nfr-design/nfr-design-patterns.md` (reuse applicable patterns)
  - `aidlc-docs/construction/core-foundation/nfr-design/logical-components.md` (reuse applicable components)
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## NFR Design Checklist
- [x] Define resilience/performance patterns for dynamic balance calculation — COMPLETED
- [x] Define ownership enforcement pattern for data isolation — COMPLETED
- [x] Define scalability patterns for account listing with pagination — COMPLETED
- [x] Define security patterns for account data access — COMPLETED
- [x] Define observability patterns (logging, metrics for account operations) — COMPLETED
- [x] Define logical components and integration boundaries — COMPLETED
- [x] Generate `accounts-nfr-design-patterns.md`
- [x] Generate `accounts-logical-components.md`
- [x] Validate design consistency against approved NFR requirements — COMPLETED
- [x] Request approval to proceed to Infrastructure Design

## Design Decisions

### Decision 1: Balance Calculation Cache Strategy

How should the `current_balance` cache be refreshed when transactions affect an account?

A) Synchronous SQL `UPDATE accounts SET current_balance = ...` in the same transaction as the journal insert
B) Asynchronous background job (eventual consistency, higher read performance)
C) Fully dynamic at query time (no cache column, always compute from journal sum)
D) Hybrid: stored `current_balance` + materialized view for historical dates

**[Answer]**: A — Synchronous atomic update in the same transaction. This ensures read-after-write consistency for `current_balance` without external job infrastructure. The `initial_balance` field provides the baseline; transaction sums are applied atomically via SQL triggers or application-level dual-write within the same DB transaction.

Rationale:
- Meets NFR-ACCT-03 (atomicity) by design.
- P95 listing latency target (50ms, NFR-ACCT-01) is achievable because the cached column avoids repeated aggregation.
- Historical date queries (NFR-ACCT-02, 100ms) may use a separate materialized snapshot approach in Phase 2.

### Decision 2: Soft Delete Implementation

How should soft-deleted accounts be excluded from queries?

A) `WHERE deleted_at IS NULL` in every query (application-level filtering)
B) PostgreSQL row-level security (RLS) policy
C) Separate `active_accounts` view that filters `deleted_at IS NULL`
D) App-level filter in repository impl

**[Answer]**: D — App-level filter in repository implementation. All `AccountReadRepository` methods implicitly append `AND deleted_at IS NULL`. Admin/internal queries (future) can use a separate method `list_all_including_deleted()`.

Rationale:
- Consistent with Core Foundation's read/write repository segregation pattern (Pattern S-01).
- Simpler to implement and test than RLS or database views.
- Matches the existing `list_by_user` repository pattern.

### Decision 3: Dynamic Balance Calculation for Historical Dates

How should `date`-filtered balance queries be implemented for Phase 1?

A) Compute dynamically: `initial_balance + SUM(journals.amount WHERE date <= $date)` at query time
B) Pre-compute and store daily balance snapshots in a separate table
C) Defer `date` parameter support to Phase 2; only return current balance in Phase 1

**[Answer]**: C — Defer `date` parameter support to Phase 2. Phase 1 returns only the current `current_balance` from the cached column. The `date` query parameter is accepted but returns current balance if provided, with documented limitations.

Rationale:
- Keeps Phase 1 implementation simple and within the 50ms p95 SLA.
- Historical balance computation requires transaction journal module integration, which is a separate UOW.
- Avoids premature optimization of a feature not yet needed by client adapters.

### Decision 4: Account Name Uniqueness Enforcement

How should duplicate account names (same user, not soft-deleted) be prevented?

A) Database UNIQUE constraint on `(user_id, name)` — hard enforcement, but conflicts with soft-delete
B) Application-level check (`SELECT COUNT(*)` before INSERT/UPDATE) — soft, race-condition window
C) Partial unique index: `CREATE UNIQUE INDEX ... WHERE deleted_at IS NULL` — hybrid approach
D) Application-level check within a serializable transaction

**[Answer]**: C — Partial unique index on `(user_id, name) WHERE deleted_at IS NULL`. This enforces uniqueness at the database level for non-deleted accounts while allowing soft-deleted accounts to retain their original name.

Rationale:
- Meets NFR-ACCT-09 (duplicate name rejection) at the database level.
- Compatible with the soft-delete hybrid policy (Decision 2).
- Firefly-III behaviour: duplicate account names are allowed across different types but not within the same type. Utopia simplifies to user-level uniqueness (regardless of type) in Phase 1, with per-type refinement deferred to Phase 2.

### Decision 5: Input Validation Architecture

Where should input validation (name length, IBAN format, type validation) be implemented?

A) Handler level only (deserialization + manual checks in handler functions)
B) Domain service level only (validate in `AccountService` before calling repository)
C) Both: basic structural validation at handler, business rule validation in service
D) Dedicated validation layer (validator structs or builder pattern)

**[Answer]**: C — Layered validation. Basic type/format checks at the handler boundary (deserialization), business rule validation (uniqueness, account_role for asset type) in `AccountService`. Error messages must follow Firefly‑III format per the error contract (§6 of functional design).

Rationale:
- Matches Core Foundation's separation of concerns.
- Ensures validation error responses are Firefly-compatible.
- IBAN format validation uses a lightweight regex (NFR-ACCT-08).

### Decision 6: Audit Logging Implementation

How should account CRUD modifications be logged (NFR-ACCT-04)?

A) Structured `tracing::info!` log events with event type, user_id, account_id, and timestamp
B) Dedicated `audit_log` database table
C) External audit service call
D) Core Foundation's existing Audit Logger component, extended with account-specific event types

**[Answer]**: D — Extend Core Foundation's Audit Logger (defined in `logical-components.md`). Account operations emit structured audit events through the same audit logger component, with new event types: `account_created`, `account_updated`, `account_deleted`, `account_restored`.

Rationale:
- Reuses existing infrastructure (Pattern SEC-03 from Core Foundation).
- Consistent security event envelope across all modules.
- No additional persistence layer needed in Phase 1.

## Derived Constraints

| Constraint | Source Decision | Implication |
|---|---|---|
| `current_balance` is synchronously updated within the transaction | Decision 1 | Service layer must accept a `&mut Transaction` parameter for write operations |
| `deleted_at IS NULL` filter is applied in repository impl | Decision 2 | Must verify all existing `AccountReadRepository` queries include this filter |
| `date` filter is deferred to Phase 2 | Decision 3 | Document in API spec that `date` parameter is accepted but not yet functional |
| Partial unique index on `(user_id, name)` | Decision 4 | Create migration to add this index alongside the schema changes |
| Layered validation in handler + service | Decision 5 | Handler validates types/formats; Service validates business rules |
| Audit via Core Foundation Audit Logger | Decision 6 | Requires Audit Logger interface to be extended with new event types |

## Approval

All planning questions have been answered. The NFR Design patterns and logical components documents have been generated:

- `aidlc-docs/construction/core-foundation/nfr-design/accounts-nfr-design-patterns.md` — 8 patterns across Data Integrity, Security, Performance, Observability, and Quality categories
- `aidlc-docs/construction/core-foundation/nfr-design/accounts-logical-components.md` — 11 components with contracts, integration topology, and operational signals

**Total artifacts delivered**: 2 documents, 8 design patterns, 11 logical components.

Next stage: **Infrastructure Design** — define database schema changes, index strategy, and deployment configuration for the Accounts Module.

[Answer]: Awaiting user approval to proceed to Infrastructure Design.

# NFR Design Patterns - Accounts Module (UOW-02)

## Scope and Design Intent
This document translates approved NFR requirements for the Accounts Module into concrete design patterns for data integrity, ownership enforcement, performance, and auditability.

## Selected Pattern Decisions (from approved answers)
- Balance cache: synchronous atomic update within the same DB transaction.
- Soft delete: app-level `deleted_at IS NULL` filter in repository (not RLS or views).
- `date` balance query: deferred to Phase 2; Phase 1 returns current balance only.
- Name uniqueness: partial unique index `(user_id, name) WHERE deleted_at IS NULL`.
- Ownership enforcement: implicit user_id filter in all repository queries.
- Validation: layered — handler (structural) + service (business rule).
- Audit: reuse Core Foundation Audit Logger with new event types.

---

## Data Integrity Patterns

### Pattern DI-01: Synchronous Balance Cache Update
- `current_balance` column is updated atomically within the same DB transaction that creates/updates/deletes a transaction journal.
- The update formula: `current_balance = initial_balance + SUM(relevant journal amounts)`.
- For account creation with `opening_balance`: the `initial_balance` column is set and an "Opening balance" transaction journal is created in the same transaction.

```
BEGIN;
  INSERT INTO accounts (...) VALUES (...) RETURNING id;
  INSERT INTO transaction_journals (account_id, amount, ...) VALUES (...);
  UPDATE accounts SET current_balance = initial_balance + <opening_amount>;
COMMIT;
```

Design rationale:
- Meets NFR-ACCT-01 (50ms listing) by eliminating the need for aggregation on every read.
- Meets NFR-ACCT-03 (atomicity) by never updating balance independently of the source journal.

### Pattern DI-02: Partial Unique Index for Account Names
- A partial unique index prevents duplicate active account names per user.
- Soft-deleted accounts are excluded from the uniqueness constraint.

```sql
CREATE UNIQUE INDEX idx_accounts_user_id_name_active
    ON accounts (user_id, LOWER(name))
    WHERE deleted_at IS NULL;
```

Design rationale:
- Meets NFR-ACCT-09 (duplicate name rejection) with database-level enforcement.
- Compatible with the soft-delete hybrid policy: deleted accounts do not block reuse of the name.

---

## Security Patterns

### Pattern SEC-ACCT-01: Implicit Ownership Filter
- Every `AccountReadRepository` method includes `user_id = $current_user_id` in the WHERE clause.
- Cross-user access cannot be expressed through the repository API; attempting to access another user's account always returns empty/None.
- The handler layer maps empty results to `404 Not Found`.

Design rationale:
- Meets NFR-ACCT-06 (cross-user access returns 404).
- Eliminates the risk of accidentally omitting the ownership check.
- Consistent with Firefly‑III behaviour (resource enumeration prevention).

### Pattern SEC-ACCT-02: Soft-Delete Visibility
- All standard repository read methods filter `deleted_at IS NULL`.
- Only explicitly named methods (e.g., `find_by_id_including_deleted`) can return soft-deleted accounts.
- Soft-deleted accounts cannot be modified (update/delete returns 404).

Design rationale:
- Meets NFR-ACCT-05 (soft-deleted accounts excluded by default).
- Prevents accidental resurrection or modification of deleted data.

---

## Performance Patterns

### Pattern P-ACCT-01: Cached Balance Column
- `current_balance` is a pre-computed column, not a live aggregation.
- Update cost is paid at write time (transaction creation/update/delete), not at read time.
- The column is indexed for range queries (future `balance_min`/`balance_max` filters).

Design rationale:
- Meets NFR-ACCT-01 (50ms p95 listing) and NFR-ACCT-02 (100ms p95 for historical queries in Phase 2).
- Read-heavy workload (listings >> creations) benefits from cached balance.

### Pattern P-ACCT-02: Paginated Listing with Offset
- Account listing uses `LIMIT + OFFSET` pagination with a `count(*)` window query for total.
- Index exists on `(user_id, name, id)` for efficient sorting and pagination.
- The existing `idx_accounts_user_id_name` index covers this pattern.

Design rationale:
- Simple, well-understood pagination pattern for Phase 1.
- Keyset pagination (cursor-based) is deferred to Phase 2 if needed for large datasets.

---

## Observability Patterns

### Pattern O-ACCT-01: Account CRUD Audit Events
- Every account modification emits a structured audit event through the Core Foundation Audit Logger.
- Event types: `account_created`, `account_updated`, `account_deleted` (soft), `account_destroyed` (hard).
- Event payload includes: `user_id`, `account_id`, `event_type`, `timestamp`, `reason_code` (for deletions).

Design rationale:
- Meets NFR-ACCT-04 (audit logging).
- Reuses existing Core Foundation Audit Logger component (Pattern SEC-03).
- Distinguishes soft vs hard deletion in audit trail.

### Pattern O-ACCT-02: Account Operation Metrics
New metric counters in the existing Prometheus namespace:
- `accounts_created_total`
- `accounts_updated_total`
- `accounts_deleted_total` (with `type` label: `soft` / `hard`)
- `accounts_listed_total`
- `accounts_balance_calculation_duration_seconds` (Phase 2)

Design rationale:
- Provides operational visibility into account module health and usage.
- Follows Core Foundation's dual-namespace metrics pattern (Pattern O-01).

---

## Quality and Testing Patterns

### Pattern Q-ACCT-01: Property-Based Testing for Account Validation
- PBT is applied to:
  - `format_amount` / `DecimalAmount` serialization round-trips (extending Core Foundation Q-01).
  - IBAN format validation (lightweight regex).
  - Account type string normalization.
- Per NFR: Property-Based Testing = Partial; validation helpers are in-scope, full integration properties are deferred.

### Pattern Q-ACCT-02: Contract Snapshot for Account Responses
- Snapshot tests capture the full `FireflyAccountResource` JSON response structure.
- Any change to the attribute set (Phase 1→2→3 attribute additions) triggers a snapshot review.
- Extends Core Foundation Pattern Q-02 (Contract Snapshot Baseline).

---

## Deferred Patterns
- Historical balance computation with `date` parameter (deferred to Phase 2).
- Keyset (cursor) pagination (deferred to Phase 2).
- Materialized balance snapshots for time-series queries (deferred to Phase 2).
- Account restore endpoint (deferred indefinitely until explicitly requested).
- Per-type account name uniqueness (deferred to Phase 2).

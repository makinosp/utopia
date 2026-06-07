# Infrastructure Design Plan - Transactions Module (UOW-03)

## Objective

Define the concrete database schema changes, index strategy, and deployment
configuration required to support the Transactions Module functional and NFR
design.

## Context Inputs

- Unit: UOW-03 Transactions Module
- Source artifacts:
  - `aidlc-docs/construction/plans/transactions-functional-design-plan.md`
  - `aidlc-docs/construction/plans/transactions-nfr-requirements-plan.md`
  - `aidlc-docs/construction/plans/transactions-nfr-design-plan.md`
  - `aidlc-docs/construction/core-foundation/infrastructure-design/infrastructure-design.md`
    (reuse)
  - `migrations/0001_initial_schema.sql` (users table)
  - `migrations/0002_accounts_schema.sql` (accounts base table)
  - `migrations/0003_accounts_extended_schema.sql` (accounts extended columns)
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## Infrastructure Design Checklist

- [x] Define transaction_journals table schema — COMPLETED
- [x] Define index strategy (listing performance, per-account queries) —
      COMPLETED
- [x] Define migration ordering and backward compatibility — COMPLETED
- [x] Define deployment configuration updates — COMPLETED
- [x] Generate migration file `0004_transactions_schema.sql`
- [x] Validate design consistency against approved functional and NFR design —
      COMPLETED
- [x] Request approval to proceed to Code Generation

## Design Decisions

### Decision 1: Single Journal Table vs. Split Transactions/Splits

Should Phase 1 use a single transaction_journals table or split into
transactions + splits tables (mirroring Firefly-III's model)?

A) Single `transaction_journals` table with `group_id` column for grouping
related splits B) Two tables: `transactions` (header) and `transaction_splits`
(individual legs) C) Single flat table without grouping support

**[Answer]**: A — Single `transaction_journals` table with a `group_id` column.
Each journal record represents a single split (one leg of a transaction).
Records sharing the same `group_id` represent related splits (e.g., a transfer
with source and destination legs). This is a simplified model that captures
Firefly-III's transaction group semantics without the complexity of a full
split-table design.

Rationale:

- Simplest schema that supports the required transaction types
  (withdrawal/deposit/transfer).
- `group_id` allows grouping related journal entries without a separate table.
- Future migration to a full split model (if needed) can use the `group_id` as
  the foreign key.

### Decision 2: Foreign Key Constraints

Should `source_id` and `destination_id` have foreign key constraints to
`accounts(id)`?

A) Yes — `REFERENCES accounts(id)` with `ON DELETE SET NULL` for referential
integrity B) Yes — `REFERENCES accounts(id)` with `ON DELETE CASCADE` C) No —
keep as loose UUID references without FK constraints

**[Answer]**: A — `REFERENCES accounts(id) ON DELETE SET NULL`. If an account is
deleted (hard or soft), the transaction journal retains the record but the
account reference becomes null. This preserves transaction history integrity.

Note: The migration `0004_transactions_schema.sql` correctly uses
`ON DELETE SET NULL` on both `source_id` and `destination_id` foreign keys,
preserving transaction history integrity when accounts are deleted.

### Decision 3: CHECK Constraint for Amount

Should the `amount` column have a CHECK constraint ensuring positive values?

A) Yes — `CHECK (amount > 0)` at the database level B) No — validate only at
application level (Transaction Service) C) Add CHECK constraint in Phase 2

**[Answer]**: B — Application-level validation only (already implemented in
`TransactionService::create_transaction`). Database CHECK constraints on
`NUMERIC` columns can cause migration issues with existing data and the
application-level validation (NFR-TXN-08) is sufficient for Phase 1.

### Decision 4: amount NOT NULL Constraint Review

Should the amount column enforce NOT NULL at the database level?

A) Yes — amount is always required for any transaction journal B) No — allow
NULL for future extensibility

**[Answer]**: A — `amount NUMERIC(20, 8) NOT NULL`. Every transaction journal
must have an amount. The current migration already enforces this.

---

## Database Schema

### New Table: `transaction_journals`

```sql
CREATE TABLE IF NOT EXISTS transaction_journals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID NOT NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('withdrawal', 'deposit', 'transfer')),
    description TEXT NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    currency_code TEXT NOT NULL,
    date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    destination_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    category_name TEXT,
    notes TEXT,
    reconciled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Update to Existing Migration

---

## Index Strategy Summary

| Index Name                    | Columns                                | Purpose                           |
| ----------------------------- | -------------------------------------- | --------------------------------- |
| `idx_tj_user_id_date` (new)   | `(user_id, date DESC)`                 | Primary listing with date sorting |
| `idx_tj_source_id` (new)      | `(user_id, source_id, date DESC)`      | Per-account source queries        |
| `idx_tj_destination_id` (new) | `(user_id, destination_id, date DESC)` | Per-account destination queries   |
| `idx_tj_group_id` (new)       | `(user_id, group_id)`                  | Group lookup                      |

All indexes include `user_id` as the leading column to support ownership-scoped
queries and partition alignment for future partitioning.

---

## Trigger: Updated At Timestamp

An `UPDATE` trigger automatically maintains `updated_at` on row modification,
consistent with the accounts table pattern from UOW-02.

---

## Deployment Configuration

No changes to the existing Docker Compose topology. The Transactions Module runs
in the same `utopia-api` container. No new environment variables or services are
required.

### Migration Order

1. `0001_initial_schema.sql` — Users, tokens, bootstrap keys (existing)
2. `0002_accounts_schema.sql` — Accounts base table (existing)
3. `0003_accounts_extended_schema.sql` — Accounts extended columns + indexes
   (existing)
4. `0004_transactions_schema.sql` — Transaction journals table + indexes (new)

### Environment Variables

No new environment variables required for Transactions Module in Phase 1.

---

## Generated Artifacts

| Artifact              | Path                                                                                   |
| --------------------- | -------------------------------------------------------------------------------------- |
| Migration file        | `migrations/0004_transactions_schema.sql`                                              |
| Infrastructure design | `aidlc-docs/construction/plans/transactions-infrastructure-design-plan.md` (this file) |

## Approval

All planning questions have been answered. The following artifacts are in place:

| Artifact              | Path                                                                       |
| --------------------- | -------------------------------------------------------------------------- |
| Migration file        | `migrations/0004_transactions_schema.sql`                                  |
| Infrastructure design | `aidlc-docs/construction/plans/transactions-infrastructure-design-plan.md` |

**Total**: 1 new table (`transaction_journals`), 4 new indexes, 1 trigger, 0 new
services/containers.

Next stage: **Code Generation** — verify existing code against the design
decisions and implement any missing pieces.

[Answer]: Awaiting user approval to proceed to Code Generation.

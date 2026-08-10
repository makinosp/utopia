# Infrastructure Design Plan - Accounts Module (UOW-02)

## Objective
Define the concrete database schema changes, index strategy, and deployment configuration required to support the Accounts Module functional and NFR design.

## Context Inputs
- Unit: UOW-02 Accounts Module
- Source artifacts:
  - `aidlc-docs/construction/plans/accounts-functional-design-plan.md`
  - `aidlc-docs/construction/plans/accounts-nfr-requirements-plan.md`
  - `aidlc-docs/construction/core-foundation/nfr-design/accounts-nfr-design-patterns.md`
  - `aidlc-docs/construction/core-foundation/nfr-design/accounts-logical-components.md`
  - `aidlc-docs/construction/core-foundation/infrastructure-design/infrastructure-design.md` (reuse)
  - `migrations/0002_accounts_schema.sql` (existing accounts table)
  - `migrations/0001_initial_schema.sql` (existing users/tokens tables)
- Extension constraints:
  - Security Baseline: Enabled
  - Property-Based Testing: Partial

## Infrastructure Design Checklist
- [x] Define accounts table schema changes (new columns for expanded attributes) — COMPLETED
- [x] Define currencies reference table — COMPLETED
- [x] Define index strategy (uniqueness, filtering, performance) — COMPLETED
- [x] Define migration ordering and backward compatibility — COMPLETED
- [x] Define deployment configuration updates — COMPLETED
- [x] Generate migration file `0003_accounts_extended_schema.sql`
- [x] Generate infrastructure design artifact `accounts-infrastructure-design.md`
- [x] Validate design consistency against approved functional and NFR design — COMPLETED
- [x] Request approval to proceed to Code Generation

## Design Decisions

### Decision 1: Currencies Table

Should a separate `currencies` reference table be introduced?

A) Yes — create a `currencies` table with `id, code, name, symbol, decimal_places` and reference it from `accounts.currency_id`
B) No — keep `currency_code` as a string column only; resolve symbol/decimal_places from a hardcoded lookup
C) Both — create `currencies` table for Phase 2, keep `currency_code` as string in Phase 1

**[Answer]**: B — No separate currencies table in Phase 1. `currency_code` remains a string column. `currency_symbol`, `currency_name`, and `currency_decimal_places` are derived from a hardcoded lookup in application code (matching the most common currencies: JPY, USD, EUR, GBP, etc.). The full `currencies` table is deferred to Phase 2 where multiple currency support and exchange rates become relevant.

Rationale:
- Avoids premature normalization for a reference table that would have minimal rows.
- Keeps Phase 1 schema changes minimal.
- The NFR Design (Decision 3) already defers `date`-based balance queries; full currency support naturally aligns with Phase 2.

### Decision 2: Migration Strategy

How should the schema migration be structured?

A) Modify `0002_accounts_schema.sql` in-place (alter existing migration)
B) Create a new `0003_accounts_extended_schema.sql` with ALTER TABLE statements
C) Create a new `0002_accounts_schema.sql` with the full schema (drop-and-recreate)

**[Answer]**: B — New migration `0003_accounts_extended_schema.sql` with `ALTER TABLE` statements. This preserves existing migration history for environments that have already run `0002_accounts_schema.sql`.

### Decision 3: IBAN/BIC Validation at DB Level

Should IBAN format be validated at the database level?

A) Yes — add `CHECK (iban ~ '...')` constraint
B) No — validate only at application level (Account Service)
C) Soft — add a CHECK constraint in Phase 2

**[Answer]**: B — Application-level validation only. IBAN validation rules differ by country; database-level CHECK would be too rigid. Lightweight regex validation in `AccountService` (NFR-ACCT-08) is sufficient.

### Decision 4: Primary Currency Storage

Where should the user's primary currency preference be stored?

A) Add a `primary_currency_code` column to the `users` table
B) Store in a new `user_preferences` key-value table
C) Hardcode a default (e.g., "JPY") for Phase 1

**[Answer]**: A — Add `primary_currency_code TEXT NOT NULL DEFAULT 'JPY'` to the `users` table. This provides the context-derived default currency per Design Question 4/§8. A separate preferences table can be introduced in a future UOW.

---

## Database Schema Changes

### Existing `accounts` Table — New Columns

```sql
-- New columns to add to accounts table:
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS initial_balance NUMERIC(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS initial_balance_date TIMESTAMPTZ;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS notes TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS iban TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS bic TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS account_number TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS virtual_balance NUMERIC(20, 8) NOT NULL DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS include_net_worth BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS "order" INTEGER;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS account_role TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS liability_type TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS liability_direction TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS interest TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS interest_period TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS cc_type TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS cc_monthly_payment_date TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS opening_balance_date TIMESTAMPTZ;
```

### New Indexes

```sql
-- Partial unique index for active account names (Pattern DI-02)
CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_user_id_name_active
    ON accounts (user_id, LOWER(name))
    WHERE deleted_at IS NULL;

-- Index for soft-delete filtering
CREATE INDEX IF NOT EXISTS idx_accounts_user_id_deleted_at
    ON accounts (user_id, deleted_at);

-- Index for balance range queries (future use)
CREATE INDEX IF NOT EXISTS idx_accounts_user_id_current_balance
    ON accounts (user_id, current_balance);
```

### Users Table — New Column

```sql
ALTER TABLE users ADD COLUMN IF NOT EXISTS primary_currency_code TEXT NOT NULL DEFAULT 'JPY';
```

---

## Index Strategy Summary

| Index Name | Columns | Condition | Purpose |
|---|---|---|---|
| `idx_accounts_user_id_name` (existing) | `(user_id, name, id)` | — | Paginated listing sort order |
| `idx_accounts_user_id_type` (existing) | `(user_id, account_type)` | — | Type filter |
| `idx_accounts_user_id_name_active` (new) | `(user_id, LOWER(name))` | `WHERE deleted_at IS NULL` | Unique active names (DI-02) |
| `idx_accounts_user_id_deleted_at` (new) | `(user_id, deleted_at)` | — | Soft-delete filtering |
| `idx_accounts_user_id_current_balance` (new) | `(user_id, current_balance)` | — | Balance range queries |

---

## Deployment Configuration

No changes to the existing Docker Compose topology. The Accounts Module runs in the same `utopia-api` container. The database migration is applied at startup via the existing migration mechanism.

### Migration Order
1. `0001_initial_schema.sql` — Users, tokens, bootstrap keys (existing)
2. `0002_accounts_schema.sql` — Accounts base table (existing)
3. `0003_accounts_extended_schema.sql` — Accounts extended columns + indexes (new)

### Environment Variables
No new environment variables required for Accounts Module in Phase 1.

---

## Generated Artifacts

| Artifact | Path |
|---|---|
| Migration file | `migrations/0003_accounts_extended_schema.sql` |
| Infrastructure design | `aidlc-docs/construction/core-foundation/infrastructure-design/accounts-infrastructure-design.md` |

## Approval

All planning questions have been answered. The following artifacts have been generated:

| Artifact | Path |
|---|---|
| Migration file | `migrations/0003_accounts_extended_schema.sql` |
| Infrastructure design | `aidlc-docs/construction/core-foundation/infrastructure-design/accounts-infrastructure-design.md` |

**Total**: 16 new columns across `accounts` and `users` tables, 3 new indexes (1 partial unique), 1 trigger, 0 new services/containers.

Next stage: **Code Generation** — implement the CRUD handlers, expanded DTOs, services, and repositories in Rust.

[Answer]: Awaiting user approval to proceed to Code Generation.

# Infrastructure Design - Accounts Module (UOW-02)

## Scope
This document extends the Core Foundation infrastructure design with the database schema changes, index strategy, and migration plan required to support the Accounts Module.

## Deployment Topology

The Accounts Module does not introduce any new services or containers. All account logic runs in the existing `utopia-api` container and uses the existing PostgreSQL instance.

### Migration Order
```
migrations/
  0001_initial_schema.sql    ← Users, tokens, bootstrap keys (existing)
  0002_accounts_schema.sql   ← Accounts base table (existing)
  0003_accounts_extended.sql ← Accounts extended attributes + indexes (NEW)
```

## Database Schema

### `accounts` Table — Final Schema

| Column | Type | Constraints | Design Reference |
|---|---|---|---|
| `id` | `UUID` | PK, default `gen_random_uuid()` | — |
| `user_id` | `UUID` | FK → `users(id) ON DELETE CASCADE` | Ownership |
| `account_type` | `TEXT` | NOT NULL | §2 Account types |
| `name` | `TEXT` | NOT NULL | §3.2 Validation |
| `current_balance` | `NUMERIC(20,8)` | NOT NULL DEFAULT 0 | Cached balance |
| `currency_code` | `TEXT` | NOT NULL | §8 Currency |
| `created_at` | `TIMESTAMPTZ` | NOT NULL DEFAULT NOW() | — |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL DEFAULT NOW() | — |
| `active` | `BOOLEAN` | NOT NULL DEFAULT true | §1 #4 |
| `order` | `INTEGER` | NULL (non-asset/liability only) | §1 #5 |
| `initial_balance` | `NUMERIC(20,8)` | NOT NULL DEFAULT 0 | §4 Balance |
| `initial_balance_date` | `TIMESTAMPTZ` | NULL | §4 Balance |
| `virtual_balance` | `NUMERIC(20,8)` | NOT NULL DEFAULT 0 | §1 #27 |
| `deleted_at` | `TIMESTAMPTZ` | NULL (soft-delete) | §7 Soft delete |
| `iban` | `TEXT` | NULL | §1 #38 |
| `bic` | `TEXT` | NULL | §1 #39 |
| `account_number` | `TEXT` | NULL | §1 #37 |
| `notes` | `TEXT` | NULL | §1 #34 |
| `include_net_worth` | `BOOLEAN` | NOT NULL DEFAULT true | §1 #45 |
| `account_role` | `TEXT` | NULL (asset accounts only) | §1 #8 |
| `liability_type` | `TEXT` | NULL | §1 #41 |
| `liability_direction` | `TEXT` | NULL | §1 #42 |
| `interest` | `TEXT` | NULL | §1 #43 |
| `interest_period` | `TEXT` | NULL | §1 #44 |
| `cc_type` | `TEXT` | NULL | §1 #36 |
| `cc_monthly_payment_date` | `TEXT` | NULL | §1 #35 |
| `opening_balance_date` | `TIMESTAMPTZ` | NULL | §1 #40 |

### `users` Table — Addition

| Column | Type | Constraints | Design Reference |
|---|---|---|---|
| `primary_currency_code` | `TEXT` | NOT NULL DEFAULT 'JPY' | §8 Currency handling |

## Index Strategy

| Index | Columns | Condition | Purpose | Pattern |
|---|---|---|---|---|
| `idx_accounts_user_id_name` | `(user_id, name, id)` | — | Paginated listing sort order | P-ACCT-02 |
| `idx_accounts_user_id_type` | `(user_id, account_type)` | — | Type filter | — |
| `idx_accounts_user_id_name_active` | `(user_id, LOWER(name))` | `WHERE deleted_at IS NULL` | Unique active names | DI-02 |
| `idx_accounts_user_id_deleted_at` | `(user_id, deleted_at)` | — | Soft-delete filter | SEC-ACCT-02 |
| `idx_accounts_user_id_current_balance` | `(user_id, current_balance)` | — | Balance range queries | P-ACCT-01 |
| `idx_accounts_user_id_account_role` | `(user_id, account_role)` | `account_role IS NOT NULL` | Role-based filtering | — |

## Logical-to-Physical Mapping

| Logical Component | Physical Placement | Storage/Infrastructure |
|---|---|---|
| Account List Handler | utopia-api container | PostgreSQL via Account Read Repository |
| Account Create Handler | utopia-api container | PostgreSQL via Account Write Repository |
| Account Get Handler | utopia-api container | PostgreSQL via Account Read Repository |
| Account Update Handler | utopia-api container | PostgreSQL via Account Write Repository |
| Account Delete Handler | utopia-api container | PostgreSQL via Account Write Repository |
| Account Service | utopia-api container | In-process validation + orchestration |
| Balance Calculator | utopia-api container | In-process SQL within transaction |
| Audit Event Factory | utopia-api container | Core Foundation Audit Logger |

## Failure Behavior

Per Core Foundation patterns:
- **Database failure during account listing**: fail closed, return 500.
- **Database failure during account write**: transaction rollback, return 500.
- **Duplicate name violation**: DB constraint error maps to 422 Validation error.
- **Cross-user access**: implicit `user_id` filter returns empty → 404.

## Storage Requirements

No additional storage beyond the existing PostgreSQL volume. The new columns and indexes add minimal overhead:
- Estimated row size increase: ~200 bytes per account row (nullable text columns).
- Index overhead: 3 new indexes, each ~8KB per 100 accounts.

## Configuration

No new environment variables. The following existing variables are relevant:
- `DATABASE_URL` — existing PostgreSQL connection string
- `PRIMARY_CURRENCY` — (future) user primary currency override

## Deployment Verification

After migration `0003` is applied:
1. Verify all new columns exist: `SELECT active, initial_balance, deleted_at, ... FROM accounts LIMIT 0;`
2. Verify unique index: `INSERT INTO accounts (user_id, name, account_type, currency_code) VALUES (..., 'test', 'asset', 'JPY')` twice for same user → second should fail.
3. Verify soft-delete exclusion: `SELECT * FROM accounts WHERE deleted_at IS NULL` excludes deleted records.
4. Verify `updated_at` trigger: `UPDATE accounts SET name = 'new' WHERE id = ...` → `updated_at` changes.
5. Verify primary currency default: `INSERT INTO users (email) VALUES ('test@example.com')` → `primary_currency_code = 'JPY'`.

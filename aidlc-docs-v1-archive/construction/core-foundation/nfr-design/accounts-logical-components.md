# Accounts Module - Logical Components (UOW-02)

## Architecture Boundary
Accounts Module components are in-process extensions of the Core Foundation architecture. They reuse Core Foundation's Auth Middleware Facade, Audit Logger, Metrics Emitter, and Error Mapper while adding account-specific logic.

## Component Inventory

| Component | Responsibility | Inputs | Outputs |
|---|---|---|---|
| Account Router | Route CRUD requests to appropriate handlers | HTTP request path + method | Dispatched handler call |
| Account List Handler | Parse list query params, call service, serialize response | `AccountListRequest`, Principal | `FireflyListEnvelope<FireflyAccountResource>` |
| Account Create Handler | Parse create request body, validate, call service | JSON body, Principal | `FireflySingleEnvelope<FireflyAccountResource>` or 422 |
| Account Get Handler | Parse account ID, call service | UUID, Principal | `FireflySingleEnvelope<FireflyAccountResource>` or 404 |
| Account Update Handler | Parse update body + ID, call service | UUID, JSON body, Principal | `FireflySingleEnvelope<FireflyAccountResource>` or 404/422 |
| Account Delete Handler | Parse ID + optional `move_to`, call service | UUID, optional `move_to`, Principal | 204 or 404 |
| Account Service | Business logic: validation, ownership check, balance update coordination | Commands, Principal | Domain objects or errors |
| Account Read Repository | Account query access with implicit ownership + soft-delete filtering | Query filter, User ID | `Paginated<AccountRecord>` or `Option<AccountRecord>` |
| Account Write Repository | Account create/update/delete with atomic balance management | Account data, Transaction | Write result |
| Balance Calculator | Compute `current_balance = initial_balance + SUM(journal amounts)` | Account ID, Transaction scope | Updated balance |
| Audit Event Factory | Create structured audit events for account operations | Event type, context fields | Audit event payload |

## Integration Topology (Text)
1. Request enters **Account Router** (registered in `build_router`).
2. Router dispatches to appropriate **Handler**.
3. Handler parses input and calls **Account Service**.
4. **Account Service** performs business rule validation.
5. For read operations: Service calls **Account Read Repository** → returns domain objects → Handler serializes to Firefly envelope.
6. For write operations: Service opens a DB transaction → calls **Account Write Repository** + **Balance Calculator** → commits → Handler serializes result.
7. All success/failure paths emit audit events via **Audit Event Factory** → Core Foundation Audit Logger.
8. All failures are mapped through Core Foundation's **Error Mapper**.

## Component Contracts

### Account Service Contract
- Must verify ownership (user_id match) on every operation.
- Must apply `deleted_at IS NULL` filter implicitly for all standard operations.
- Must wrap balance-affecting writes in a DB transaction (Pattern DI-01).
- Must return `DomainError::NotFound` for missing/unowned accounts.
- Must return `DomainError::Validation` with field-level errors for invalid input.

### Account Read Repository Contract
- All query methods implicitly append `AND user_id = $1 AND deleted_at IS NULL`.
- `list_by_user` returns `Paginated<AccountRecord>` with total count via window query.
- `find_by_id(user_id, account_id)` returns `Option<AccountRecord>`, never cross-user data.

### Account Write Repository Contract
- `create` accepts a `&mut Transaction` parameter and account data; returns the created record.
- `update` accepts a `&mut Transaction` parameter, account ID, and partial update data; returns updated record.
- `soft_delete` sets `deleted_at = NOW()` within a transaction.
- `hard_delete` performs `DELETE FROM accounts WHERE id = $1` (only when no transactions exist).
- `reassign_transactions` moves transactions to a target account before deletion.

### Balance Calculator Contract
- Computes `current_balance = initial_balance + journal_sum` for a given account.
- The journal_sum is calculated within the same transaction to avoid race conditions.
- For Phase 1, always uses the full transaction history (no date filtering).

## Non-Functional Mapping Matrix

| NFR Requirement | Primary Component(s) | Supporting Component(s) |
|---|---|---|
| NFR-ACCT-01 (50ms p95 listing) | Account Read Repository | Account List Handler, Pagination utilities |
| NFR-ACCT-02 (100ms balance calc) | Balance Calculator | Account Write Repository |
| NFR-ACCT-03 (atomic creation) | Account Write Repository, Balance Calculator | Account Service |
| NFR-ACCT-04 (audit logging) | Audit Event Factory | Core Foundation Audit Logger |
| NFR-ACCT-05 (soft-delete visibility) | Account Read Repository | Account Service |
| NFR-ACCT-06 (cross-user 404) | Account Read Repository | Account Service, Account Get/Update/Delete Handlers |
| NFR-ACCT-07 (name max 255) | Account Service (validation) | Account Create/Update Handlers |
| NFR-ACCT-08 (IBAN validation 5ms) | Account Service (validation) | Lightweight regex validator |
| NFR-ACCT-09 (duplicate name rejection) | Partial Unique Index (DB) | Account Write Repository |

## Operational Signals by Component

| Component | Must Emit |
|---|---|
| Account List Handler | `accounts_listed_total` metric counter |
| Account Create Handler | `accounts_created_total` metric + `account_created` audit event |
| Account Update Handler | `accounts_updated_total` metric + `account_updated` audit event |
| Account Delete Handler | `accounts_deleted_total` metric + `account_deleted` / `account_destroyed` audit event |
| Account Service | `account_validation_error_total` metric (by failure reason) |
| Balance Calculator | `account_balance_calculation_duration_seconds` (Phase 2) |

## Extension of Core Foundation Components

| Core Foundation Component | Accounts Module Extension |
|---|---|
| Error Mapper | Add `DomainError::AccountNotFound` → 404 mapping |
| Metrics Emitter | Add `accounts_*` metric namespace |
| Audit Logger | Add `account_*` event types |
| Repository Read/Write Interfaces | Add `AccountReadRepository` and `AccountWriteRepository` traits |

## Deferred Componentization
- No separate balance snapshot materialization service (Phase 2).
- No account search/indexing service (Phase 2).
- No bulk account operations service (Phase 2).
- No account reconciliation service (Phase 2).

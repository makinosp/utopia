# Functional Design Plan - Transactions Module (UOW-03)

## Objective
Define the detailed business logic, schema attributes, validation rules, ownership rules, and compatibility mappings for the Transactions Module (UOW-03).

## Unit Context
- **Unit Name**: Transactions Module (UOW-03)
- **Primary Stories**:
  - **US-006**: List Transactions
  - **US-007**: Get Single Transaction
  - **US-008**: Create Transaction
  - **US-009**: Update Transaction
  - **US-010**: Delete Transaction
  - **US-011**: List Account Transactions
- **Key Responsibilities**:
  - CRUD operations for transaction journals.
  - Per-account transaction listing.
  - Atomic balance updates on accounts when transactions are created/updated/deleted.
  - Strict ownership checks: transactions are bound to the authenticated user principal.
  - Read-only dependency on UOW-02 Accounts for existence and ownership verification.

---

## 1. Transaction Data Model

### Core Design Decisions

1. **Simplified single-table model**: Phase 1 uses a single `transaction_journals` table with a `group_id` column to group related splits (matching Firefly‑III's transaction_group concept at a basic level).
2. **Transaction types**: `withdrawal`, `deposit`, `transfer` — matching Firefly‑III's primary types.
3. **Balance updates**: Each transaction write operation atomically recalculates the affected accounts' `current_balance`.
4. **Source/Destination semantics**:
   - `withdrawal`: source = asset account (money leaves), destination = expense account (optional)
   - `deposit`: source = revenue account (optional), destination = asset account (money arrives)
   - `transfer`: source = asset account, destination = asset account

### Table Schema

```sql
CREATE TABLE transaction_journals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID NOT NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('withdrawal', 'deposit', 'transfer')),
    description TEXT NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    currency_code TEXT NOT NULL,
    date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_id UUID REFERENCES accounts(id),
    destination_id UUID REFERENCES accounts(id),
    category_name TEXT,
    notes TEXT,
    reconciled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Indexes

```sql
-- Primary listing: by user with date ordering
CREATE INDEX idx_tj_user_id_date ON transaction_journals (user_id, date DESC);

-- Per-account listing: find all transactions involving a specific account
CREATE INDEX idx_tj_source_id ON transaction_journals (user_id, source_id, date DESC);
CREATE INDEX idx_tj_destination_id ON transaction_journals (user_id, destination_id, date DESC);

-- Group lookup
CREATE INDEX idx_tj_group_id ON transaction_journals (user_id, group_id);
```

---

## 2. API Endpoints Specification

### 2.1 `GET /api/v1/transactions` — List Transactions

**Query Parameters**:

| Parameter | Type | Default | Description |
|---|---|---|---|
| `page` | int | 1 | Page number (1-based) |
| `limit` | int | 50 | Items per page (max 100) |
| `start` | string (ISO 8601) | — | Filter by start date |
| `end` | string (ISO 8601) | — | Filter by end date |
| `type` | string | — | Filter by transaction type |

**Response**: `FireflyListEnvelope` with `data: [FireflyTransactionResource]` and `meta.pagination`.

**Status Codes**: `200 OK` on success; `401 Unauthorized`; `422 Unprocessable Entity`; `500 Server Error`.

### 2.2 `POST /api/v1/transactions` — Create Transaction

**Request Body** (JSON):

```json
{
  "group_id": "uuid-or-null-for-auto",
  "transaction_type": "withdrawal",
  "description": "Groceries",
  "amount": "45.50",
  "currency_code": "JPY",
  "date": "2026-05-29T00:00:00Z",
  "source_id": "source-account-uuid",
  "destination_id": "dest-account-uuid",
  "category_name": "Food",
  "notes": "Weekly groceries",
  "reconciled": false
}
```

**Validation Rules**:
- `transaction_type`: Required, one of `withdrawal`, `deposit`, `transfer`
- `description`: Required, string, max 255 chars
- `amount`: Required, positive decimal string
- `currency_code`: Required, valid currency code
- `date`: Required, ISO 8601 timestamp
- `source_id`: Required if type is `withdrawal` or `transfer`; destination for `deposit`
- `destination_id`: Required if type is `deposit` or `transfer`; source for `withdrawal`
- Source and destination must:
  - Exist and belong to the authenticated user
  - Be different for `transfer` type

**Response**: `FireflySingleEnvelope` with `data: FireflyTransactionResource`.

**Status Codes**: `201 Created` on success; `422 Unprocessable Entity` on validation failure.

### 2.3 `GET /api/v1/transactions/:id` — Get Transaction

**Path Parameters**: `id` — UUID of the transaction.

**Response**: `FireflySingleEnvelope` with `data: FireflyTransactionResource`.

**Status Codes**: `200 OK` on success; `404 Not Found` if missing or unauthorized.

### 2.4 `PUT /api/v1/transactions/:id` — Update Transaction

**Request Body**: Same structure as POST, all fields optional (partial update).

**Additional Validation for Amount Change**: When the amount changes, both source and destination account balances are recalculated.

**Response**: `FireflySingleEnvelope` with `data: FireflyTransactionResource`.

**Status Codes**: `200 OK` on success; `422 Unprocessable Entity` on validation failure; `404 Not Found` if missing.

### 2.5 `DELETE /api/v1/transactions/:id` — Delete Transaction

**Behavior**: Hard delete from the database. Before deletion, account balances are atomically reversed.

**Status Codes**: `204 No Content` on success; `404 Not Found` if missing or unauthorized.

### 2.6 `GET /api/v1/accounts/:id/transactions` — List Account Transactions

**Path Parameters**: `id` — UUID of the account.

**Query Parameters**:

| Parameter | Type | Default | Description |
|---|---|---|---|
| `page` | int | 1 | Page number (1-based) |
| `limit` | int | 50 | Items per page (max 100) |

**Behavior**: Returns all transactions where the account is either source or destination, ordered by date descending.

**Response**: `FireflyListEnvelope` with `data: [FireflyTransactionResource]` and `meta.pagination`.

**Status Codes**: `200 OK` on success; `404 Not Found` if account does not exist or does not belong to user.

---

## 3. Firefly‑III Transaction Response Schema

The following table defines the Firefly‑III `TransactionTransformer` response attributes that Utopia must serialize.

| # | Attribute | Type | Nullable | Utopia Implementation |
|---|---|---|---|---|
| 1 | `id` | `string` | No | `Uuid::to_string()` |
| 2 | `created_at` | `string` (Atom) | No | `DateTime<Utc>::to_rfc3339()` |
| 3 | `updated_at` | `string` (Atom) | No | `DateTime<Utc>::to_rfc3339()` |
| 4 | `user` | `string` | No | User email (resolved) |
| 5 | `group_id` | `string` | No | `group_id.to_string()` |
| 6 | `type` | `string` | No | `transaction_type` |
| 7 | `date` | `string` (Atom) | No | `DateTime<Utc>::to_rfc3339()` |
| 8 | `description` | `string` | No | `String` |
| 9 | `amount` | `string` | No | `DecimalAmount` formatted |
| 10 | `currency_code` | `string` | No | `String` |
| 11 | `source_id` | `string` | Yes | `Option<Uuid>::to_string()` |
| 12 | `source_name` | `string` | Yes | Account name resolved from ID |
| 13 | `destination_id` | `string` | Yes | `Option<Uuid>::to_string()` |
| 14 | `destination_name` | `string` | Yes | Account name resolved from ID |
| 15 | `category_name` | `string` | Yes | `Option<String>` |
| 16 | `notes` | `string` | Yes | `Option<String>` |
| 17 | `reconciled` | `bool` | No | `bool` |
| 18 | `links` | `array` | No | HATEOAS self-link |

---

## 4. Balance Update Strategy

### 4.1 Account Balance Rules

- **Withdrawal**: `source.current_balance -= amount` (money leaves source)
- **Deposit**: `destination.current_balance += amount` (money arrives at destination)
- **Transfer**: `source.current_balance -= amount`, `destination.current_balance += amount`

### 4.2 Atomic Operations

All write operations that affect balances MUST:
1. Begin a database transaction
2. Perform the transaction journal CRUD operation
3. Atomically update affected account balances
4. Commit the database transaction

### 4.3 Balance Reversal on Delete

When a transaction is deleted, the balance effects are reversed:
- **Withdrawal**: `source.current_balance += amount`
- **Deposit**: `destination.current_balance -= amount`
- **Transfer**: `source.current_balance += amount`, `destination.current_balance -= amount`

---

## 5. Ownership Enforcement

- Every transaction is strictly bound to `user_id` via the principal context.
- All CRUD operations filter by the authenticated user's ID.
- Account existence checks (source/destination) are scoped to the user's accounts.
- Cross-user access returns `404 Not Found` (not 403).

---

## 6. Error Scenarios

| Scenario | HTTP Status | `message` | `errors` |
|---|---|---|---|
| Transaction not found | 404 | `"Not Found"` | `{}` |
| Cross-user access | 404 | `"Not Found"` | `{}` |
| Missing description | 422 | `"The given data was invalid."` | `{"description": ["The description field is required."]}` |
| Invalid transaction type | 422 | `"The given data was invalid."` | `{"transaction_type": ["The selected type is invalid."]}` |
| Invalid amount | 422 | `"The given data was invalid."` | `{"amount": ["The amount must be a positive number."]}` |
| Source account not found | 422 | `"The given data was invalid."` | `{"source_id": ["The selected source account is invalid."]}` |
| Destination account not found | 422 | `"The given data was invalid."` | `{"destination_id": ["The selected destination account is invalid."]}` |
| Transfer with same source/destination | 422 | `"The given data was invalid."` | `{"destination_id": ["Source and destination must be different for transfers."]}` |
| Missing source for withdrawal | 422 | `"The given data was invalid."` | `{"source_id": ["The source account is required for withdrawals."]}` |
| Missing destination for deposit | 422 | `"The given data was invalid."` | `{"destination_id": ["The destination account is required for deposits."]}` |

---

## 7. Currency Handling

- Currency code is required on creation.
- No automatic currency conversion in Phase 1.
- Amounts are stored as `NUMERIC(20,8)` and serialized as string for API compatibility.

---

## 8. JSON Envelope Compatibility

- **List response**:
```json
{
  "data": [ { "type": "transactions", "id": "uuid", "attributes": {...} } ],
  "meta": { "pagination": { "total": 10, "count": 5, "per_page": 5, "current_page": 1, "total_pages": 2 } }
}
```
- **Single response**:
```json
{
  "data": { "type": "transactions", "id": "uuid", "attributes": {...} }
}
```
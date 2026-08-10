# Functional Design Plan - Accounts Module (UOW-02)

## Objective
Define the detailed business logic, schema attributes, validation rules, ownership rules, and compatibility mappings for the Accounts Module (UOW-02).

## Unit Context
- **Unit Name**: Accounts Module (UOW-02)
- **Primary Stories**:
  - **US-001**: Create Asset Account
  - **US-002**: Create Expense/Revenue/Liability Account
  - **US-003**: List Accounts (with filtering & pagination)
  - **US-004**: Update Account Details
  - **US-005**: Delete Account (including cascades/validations)
- **Key Responsibilities**:
  - CRUD operations for all supported account types.
  - Strict ownership checks: accounts are strictly bound to the authenticated user principal.
  - Firefly-III JSON envelope & attribute nesting compatibility.
  - Correct precision persistence for initial balance (if stored) and active balance calculation hooks.

---

## 1. Firefly‑III Account Response Schema (Complete Mapping)

The following table defines every attribute in the Firefly‑III `AccountTransformer` response. Utopia must serialize responses matching this structure exactly (per Design Question 5: Strict compatibility).

| # | Attribute | Type | Nullable | Source in Firefly‑III | Utopia Implementation |
|---|---|---|---|---|---|
| 1 | `id` | `string` | No | `$account->id` cast to string | `Uuid::to_string()` |
| 2 | `created_at` | `string` (Atom) | No | `$account->created_at->toAtomString()` | `DateTime<Utc>::to_rfc3339()` |
| 3 | `updated_at` | `string` (Atom) | No | `$account->updated_at->toAtomString()` | `DateTime<Utc>::to_rfc3339()` |
| 4 | `active` | `boolean` | No | `$account->active` | `bool` from DB column |
| 5 | `order` | `int` | Yes | `$account->order` (null for non-asset/liability) | `Option<i32>` |
| 6 | `name` | `string` | No | `$account->name` | `String` |
| 7 | `type` | `string` | No | `shortNamesByFullName` config lookup | Short type name (see §2) |
| 8 | `account_role` | `string` | Yes | `$account->meta['account_role']` | `Option<String>` (meta) |
| 9 | `object_group_id` | `string` | Yes | Meta value | `Option<Uuid>` (TBD: future) |
| 10 | `object_group_order` | `int` | Yes | Meta value | `Option<i32>` (TBD: future) |
| 11 | `object_group_title` | `string` | Yes | Meta value | `Option<String>` (TBD: future) |
| 12 | `object_has_currency_setting` | `bool` | No | Whether currency differs from primary | `bool` |
| 13 | `currency_id` | `string` | No | Account-specific or primary currency ID | `Uuid` or primary fallback |
| 14 | `currency_name` | `string` | No | Currency name | `String` |
| 15 | `currency_code` | `string` | No | Currency code (e.g. "JPY") | `String` (already exists) |
| 16 | `currency_symbol` | `string` | No | Currency symbol (e.g. "¥") | `String` |
| 17 | `currency_decimal_places` | `int` | No | `$currency->decimal_places` | `i32` (default 2) |
| 18 | `primary_currency_id` | `string` | No | User's primary currency ID | Resolved from user context |
| 19 | `primary_currency_name` | `string` | No | Primary currency name | Resolved from user context |
| 20 | `primary_currency_code` | `string` | No | Primary currency code | Resolved from user context |
| 21 | `primary_currency_symbol` | `string` | No | Primary currency symbol | Resolved from user context |
| 22 | `primary_currency_decimal_places` | `int` | No | Primary decimal places | Resolved from user context |
| 23 | `current_balance` | `string` | No | Enriched balance | `String` (decimal formatted) |
| 24 | `pc_current_balance` | `string` | Yes | Primary-currency converted balance | `Option<String>` |
| 25 | `opening_balance` | `string` | Yes | `initial_balance` or null | `Option<String>` |
| 26 | `pc_opening_balance` | `string` | Yes | Primary-currency converted opening | `Option<String>` |
| 27 | `virtual_balance` | `string` | Yes | Virtual/envelope balance | `Option<String>` |
| 28 | `pc_virtual_balance` | `string` | Yes | Primary-currency converted virtual | `Option<String>` |
| 29 | `debt_amount` | `string` | Yes | Debt balance (liabilities) | `Option<String>` |
| 30 | `pc_debt_amount` | `string` | Yes | Primary-currency converted debt | `Option<String>` |
| 31 | `balance_difference` | `string` | Yes | current - opening difference | `Option<String>` |
| 32 | `pc_balance_difference` | `string` | Yes | Primary-currency diff | `Option<String>` |
| 33 | `current_balance_date` | `string` (Atom) | No | Enrichment date | `DateTime<Utc>::to_rfc3339()` |
| 34 | `notes` | `string` | Yes | `$account->meta['notes']` | `Option<String>` |
| 35 | `monthly_payment_date` | `string` | Yes | Credit card payment date | `Option<String>` |
| 36 | `credit_card_type` | `string` | Yes | `$account->meta['cc_type']` | `Option<String>` |
| 37 | `account_number` | `string` | Yes | `$account->meta['account_number']` | `Option<String>` |
| 38 | `iban` | `string` | Yes | `$account->iban` (empty → null) | `Option<String>` |
| 39 | `bic` | `string` | Yes | `$account->meta['BIC']` | `Option<String>` |
| 40 | `opening_balance_date` | `string` (Atom) | Yes | `$account->meta['opening_balance_date']` | `Option<String>` |
| 41 | `liability_type` | `string` | Yes | e.g. "loan", "debt", "mortgage" | `Option<String>` |
| 42 | `liability_direction` | `string` | Yes | "credit" or "debit" | `Option<String>` |
| 43 | `interest` | `string` | Yes | Interest rate for liabilities | `Option<String>` |
| 44 | `interest_period` | `string` | Yes | e.g. "monthly", "yearly" | `Option<String>` |
| 45 | `include_net_worth` | `bool` | No | Whether account counts toward net worth | `bool` |
| 46 | `longitude` | `float` | Yes | Location data | `Option<f64>` |
| 47 | `latitude` | `float` | Yes | Location data | `Option<f64>` |
| 48 | `zoom_level` | `int` | Yes | Location data | `Option<i32>` |
| 49 | `last_activity` | `string` (Atom) | Yes | Latest transaction date | `Option<String>` |
| 50 | `links` | `array` | No | HATEOAS self-link | `[{"rel": "self", "uri": "/api/v1/accounts/{id}"}]` |

### Implementation Priority for Attributes

**Phase 1 (Immediate — Code Generation)**: #1–#7, #13–#17, #23, #25–#26, #33–#34, #38–#39, #45, #50
- These are the most commonly consumed attributes by external clients (Waterfly‑III etc.)

**Phase 2 (Soon — Accounts Module)**: #8, #18–#22, #24, #27–#32, #35–#37, #40–#44, #46–#49
- These add deeper compatibility but require additional meta-data infrastructure

**Phase 3 (Future)**: #9–#11 (object groups require a separate ObjectGroup module)

---

## 2. Supported Account Types

Based on Design Question 1 (Answer: C — Full set), Utopia will support all Firefly‑III account types. The mapping between full type names and short names follows Firefly‑III conventions:

| Short Type (API response) | Full Type Name | Category | Initial Support |
|---|---|---|---|
| `asset` | Asset account | Asset | ✅ Phase 1 |
| `expense` | Expense account | Expense | ✅ Phase 1 |
| `revenue` | Revenue account | Revenue | ✅ Phase 1 |
| `liability` | Liability account | Liability | ✅ Phase 2 |
| `liabilities` | (Alias for liability) | Liability | ✅ Phase 2 |
| `cash` | Cash account | Asset | ✅ Phase 2 |
| `loan` | Loan | Liability | ✅ Phase 2 |
| `debt` | Debt | Liability | ✅ Phase 2 |
| `mortgage` | Mortgage | Liability | ✅ Phase 2 |
| `credit card` | Credit card | Liability | ✅ Phase 2 |
| `hidden` | Hidden account | Internal | ✅ Phase 2 |
| `special` | Special account | Internal | ✅ Phase 2 |
| `default account` | Default account | Internal | ✅ Phase 2 |

### Validation Rules per Type

- **Asset accounts**: Require `account_role` (one of: `defaultAsset`, `sharedAsset`, `savingAsset`, `ccAsset`, `cashWalletCharity`)
- **Liability accounts**: Require `liability_type` and optional `interest`/`interest_period`; `liability_direction` controls credit/debit semantics
- **Expense/Revenue**: No additional required meta fields beyond name and currency
- **All types**: Name is required; IBAN is optional but validated for format when provided

---

## 3. API Endpoints Specification (Full CRUD)

### 3.1 `GET /api/v1/accounts` — List Accounts

**Query Parameters**:

| Parameter | Type | Default | Description |
|---|---|---|---|
| `page` | int | 1 | Page number (1-based) |
| `limit` | int | 50 | Items per page (max 100) |
| `type` | string | — | Filter by account type short name |
| `date` | string (ISO 8601) | — | Calculate balances as of this date (TBD Phase 2) |
| `search` | string | — | Free-text search on name/type/IBAN (TBD Phase 2) |
| `balance_min` | decimal | — | Minimum current balance filter (TBD Phase 2) |
| `balance_max` | decimal | — | Maximum current balance filter (TBD Phase 2) |

**Response**: `FireflyListEnvelope` with `data: [FireflyAccountResource]` and `meta.pagination`.

### 3.2 `POST /api/v1/accounts` — Create Account

**Request Body** (JSON):

```json
{
  "name": "My Account",
  "type": "asset",
  "currency_id": "uuid-or-null",
  "currency_code": "JPY",
  "active": true,
  "include_net_worth": true,
  "account_role": "defaultAsset",
  "iban": "optional-iban",
  "bic": "optional-bic",
  "account_number": "optional-account-number",
  "opening_balance": "1000.00",
  "opening_balance_date": "2026-01-01T00:00:00Z",
  "virtual_balance": "0",
  "notes": "optional notes",
  "liability_type": null,
  "liability_direction": null,
  "interest": null,
  "interest_period": null,
  "latitude": null,
  "longitude": null,
  "zoom_level": null
}
```

**Validation Rules**:
- `name`: Required, string, max 255 chars, unique per user (case-insensitive)
- `type`: Required, must be a valid short type name from §2
- `currency_id` OR `currency_code`: At least one must be provided; falls back to user's primary currency
- `opening_balance` and `opening_balance_date`: Both required if either is provided (creates an "Opening balance" transaction)
- `active`: Optional, default `true`
- `include_net_worth`: Optional, default `true`
- `account_role`: Required for type=`asset`; optional otherwise
- `iban`: Optional, validated with IBAN format check when provided
- `bic`: Optional, validated for format when provided

**Response**: `FireflySingleEnvelope` with `data: FireflyAccountResource`.

**Status Codes**: `201 Created` on success; `422 Unprocessable Entity` on validation failure.

### 3.3 `GET /api/v1/accounts/:id` — Get Account

**Path Parameters**: `id` — UUID of the account.

**Response**: `FireflySingleEnvelope` with `data: FireflyAccountResource`.

**Status Codes**: `200 OK` on success; `404 Not Found` if missing or belongs to another user.

### 3.4 `PUT /api/v1/accounts/:id` — Update Account

**Request Body**: Same structure as POST, all fields optional (partial update).

**Validation Rules**: Same as POST, but:
- `type` can only be changed if no transactions exist for this account
- `currency_id`/`currency_code` change is allowed but recalculates balances

**Response**: `FireflySingleEnvelope` with `data: FireflyAccountResource`.

**Status Codes**: `200 OK` on success; `422 Unprocessable Entity` on validation failure; `404 Not Found` if missing.

### 3.5 `DELETE /api/v1/accounts/:id` — Delete Account

**Behaviour** (per Design Question 3, Answer D — Hybrid policy):
- Soft delete: Sets `deleted_at` timestamp (data preserved for integrity)
- If `move_to` query parameter is provided (a target account UUID), all transactions are reassigned before deletion
- If no `move_to` specified and transactions exist, the account is soft-deleted (hidden from API but transactions remain)

**Status Codes**: `204 No Content` on success; `404 Not Found` if missing or unauthorized.

---

## 4. Balance Calculation Strategy

Per Design Question 2 (Answer: C — Both initial_balance + transaction sums):

### Data Model
```
accounts table:
  - initial_balance NUMERIC(20,8) NOT NULL DEFAULT 0   ← NEW
  - initial_balance_date TIMESTAMPTZ                     ← NEW
  - current_balance NUMERIC(20,8) NOT NULL DEFAULT 0   ← EXISTING (cache)
```

### Calculation Rules
1. **On account creation**: `initial_balance` is set from `opening_balance` input; `current_balance` is set to the same value.
2. **Transaction-based recalculation**: `current_balance = initial_balance + SUM(transaction amounts)` where transactions affect this account.
3. **Cache update**: On every transaction creation/update/deletion that involves this account, `current_balance` is atomically updated via SQL.
4. **Historical balance**: When `date` query parameter is provided, calculate `balance_at_date = initial_balance + SUM(transaction amounts up to date)` at query time.
5. **Opening balance transaction**: When an account is created with `opening_balance`, an "Opening balance" transaction journal is created internally (matching Firefly‑III behaviour).

---

## 5. Ownership Enforcement

- Every account is strictly bound to `user_id`.
- All CRUD operations filter by `WHERE user_id = $current_user_id`.
- Cross-user access returns `404 Not Found` (not 403) to prevent resource enumeration.
- The principal is extracted from the bearer token by the auth middleware.

---

## 6. Error Scenarios (Extended Error Contract)

In addition to the Core Foundation error contract, the Accounts Module defines:

| Scenario | HTTP Status | `message` | `errors` |
|---|---|---|---|
| Account not found | 404 | `"Not Found"` | `{}` |
| Cross-user access to account | 404 | `"Not Found"` | `{}` |
| Missing required `name` | 422 | `"The given data was invalid."` | `{"name": ["The name field is required."]}` |
| Invalid account type | 422 | `"The given data was invalid."` | `{"type": ["The selected type is invalid."]}` |
| Duplicate account name (same user) | 422 | `"The given data was invalid."` | `{"name": ["The name has already been taken."]}` |
| Invalid IBAN format | 422 | `"The given data was invalid."` | `{"iban": ["The iban field must be a valid IBAN."]}` |
| Opening balance without date | 422 | `"The given data was invalid."` | `{"opening_balance_date": ["The opening balance date field is required when opening balance is present."]}` |

---

## 7. Soft Delete Policy

Per Design Question 3 (Answer: D — Hybrid policy):

1. **When no transactions exist**: Hard delete from database.
2. **When transactions exist and `move_to` is provided**: Reassign all transactions to the target account, then hard delete.
3. **When transactions exist and `move_to` is NOT provided**: Soft delete (set `deleted_at`). Account is hidden from API responses but transactions remain intact.
4. **Restore**: Not implemented in Phase 1 (requires explicit restore endpoint).

---

## 8. Currency Handling

Per Design Question 4 (Answer: C — Context-derived default):

1. On creation, if `currency_id` or `currency_code` is provided, use that currency.
2. If neither is provided, resolve the user's primary currency from their preferences/context.
3. The resolved currency is persisted in `currency_code` (and `currency_id` if a currencies table exists).
4. `currency_symbol` and `currency_decimal_places` are deterministically derived from `currency_code` (e.g., via a lookup table).

---

## 9. JSON Envelope Compatibility

Per Design Question 5 (Answer: A — Strict compatibility):

- **List response**:
```json
{
  "data": [ { "type": "accounts", "id": "uuid", "attributes": {...} } ],
  "meta": { "pagination": { "total": 10, "count": 5, "per_page": 5, "current_page": 1, "total_pages": 2 } }
}
```
- **Single response**:
```json
{
  "data": { "type": "accounts", "id": "uuid", "attributes": {...} }
}
```

---

## Planning Checklist (Updated)
- [x] Analyze Firefly-III account schema compatibility (nested models) — COMPLETED (§1)
- [x] List all supported account types and validation rules per type — COMPLETED (§2)
- [x] Define the ownership enforcement mechanism within CRUD flows — COMPLETED (§5)
- [x] Formulate account cascade delete policies (handling of referenced transactions) — COMPLETED (§7)
- [x] Design pagination and filtered listing boundaries for accounts — COMPLETED (§3.1)
- [x] Draft Functional Design artifact templates for generation — COMPLETED
- [x] Request planning approval and resolve design preferences — COMPLETED

## Next Steps

1. **Review and approve** this functional design document.
2. Proceed to **NFR Requirements** for Accounts Module (UOW-02).
3. Then **NFR Design**, **Infrastructure Design**, and finally **Code Generation** (where CRUD endpoints, schema changes, and DTO expansions will be implemented).

# User Stories

## Story Format

```
As a [persona], I want to [goal] so that [benefit].
```

**Priority tags**: Must / Should / Could  
**Acceptance Criteria format**: Functional + compatibility conditions  
**NFR approach**: Critical NFRs (auth, data integrity, compatibility) inline per story; broad policy items in Cross-Cutting Constraints section.

---

## Cross-Cutting Constraints

These constraints apply to all stories below unless explicitly overridden.

- **Authentication**: All endpoints require valid bearer token authentication. Unauthenticated requests must return HTTP 401 with a Firefly-III compatible error body.
- **Authorization**: Operations must be scoped to the authenticated user's data only.
- **Error format**: All error responses must use the Firefly-III standard error response schema (`{ message, errors }` structure).
- **Content type**: All requests and responses must use `Content-Type: application/json`.
- **Data integrity**: Monetary amounts must be stored and returned as strings to preserve decimal precision (Firefly-III convention).
- **Compatibility scope**: Stories in this document target Core bookkeeping flows + common metadata endpoints.

---

## Epic: Account Management

### US-001 — List Accounts
**Priority**: Must

As Maya (Household End User), I want to retrieve a list of my accounts so that I can see an overview of all balances in my client app.

**Acceptance Criteria**:
- `GET /api/v1/accounts` returns HTTP 200 with a paginated list of account objects.
- Each account object includes: `id`, `type`, `name`, `current_balance`, `currency_code`, `created_at`, `updated_at`.
- Response schema matches the Firefly-III `AccountRead` list response format.
- Supports `?type=` query parameter to filter by account type (asset, expense, revenue, etc.).
- Pagination follows Firefly-III convention: `meta.pagination` object with `total`, `count`, `per_page`, `current_page`, `total_pages`.
- Requires valid bearer token; returns 401 if missing or invalid.

---

### US-002 — Get Single Account
**Priority**: Must

As Maya (Household End User), I want to retrieve the details of a single account so that my client app can display accurate balance and metadata for that account.

**Acceptance Criteria**:
- `GET /api/v1/accounts/{id}` returns HTTP 200 with a single account object.
- Response schema matches the Firefly-III `AccountSingle` response format.
- Returns HTTP 404 with a Firefly-III compatible error body if the account does not exist or is not owned by the authenticated user.
- Requires valid bearer token.

---

### US-003 — Create Account
**Priority**: Must

As Kenji (Self-Hosting Admin), I want to create a new account via the API so that accounts can be provisioned for household members.

**Acceptance Criteria**:
- `POST /api/v1/accounts` accepts the Firefly-III `AccountStore` request body.
- On success, returns HTTP 200 with the created account in `AccountSingle` format.
- Validates required fields (`name`, `type`); returns HTTP 422 with a Firefly-III compatible validation error body on missing or invalid input.
- Requires valid bearer token.

---

### US-004 — Update Account
**Priority**: Should

As Kenji (Self-Hosting Admin), I want to update an existing account via the API so that account metadata stays accurate over time.

**Acceptance Criteria**:
- `PUT /api/v1/accounts/{id}` accepts the Firefly-III `AccountUpdate` request body.
- On success, returns HTTP 200 with the updated account in `AccountSingle` format.
- Returns HTTP 404 if account not found or not owned by the authenticated user.
- Returns HTTP 422 on invalid input with a Firefly-III compatible validation error body.
- Requires valid bearer token.

---

### US-005 — Delete Account
**Priority**: Should

As Kenji (Self-Hosting Admin), I want to delete an account via the API so that obsolete accounts can be cleaned up.

**Acceptance Criteria**:
- `DELETE /api/v1/accounts/{id}` returns HTTP 204 on success.
- Returns HTTP 404 if account not found or not owned by the authenticated user.
- Requires valid bearer token.

---

## Epic: Transaction Management

### US-006 — List Transactions
**Priority**: Must

As Maya (Household End User), I want to retrieve a paginated list of transactions so that my client app can display my transaction history accurately.

**Acceptance Criteria**:
- `GET /api/v1/transactions` returns HTTP 200 with a paginated list of transaction group objects.
- Each transaction group matches the Firefly-III `TransactionRead` format including `transactions[]` array with splits.
- Supports `?type=` (withdrawal, deposit, transfer, etc.) and `?start=` / `?end=` date range filters.
- Pagination follows Firefly-III convention.
- Requires valid bearer token.

---

### US-007 — Get Single Transaction
**Priority**: Must

As Maya (Household End User), I want to retrieve the details of a single transaction so that my client app can display full transaction metadata.

**Acceptance Criteria**:
- `GET /api/v1/transactions/{id}` returns HTTP 200 with a single transaction group in `TransactionSingle` format.
- Returns HTTP 404 if not found or not owned by the authenticated user.
- Requires valid bearer token.

---

### US-008 — Create Transaction
**Priority**: Must

As Maya (Household End User), I want to record a new transaction so that my household budget data stays up to date as I spend or receive money.

**Acceptance Criteria**:
- `POST /api/v1/transactions` accepts the Firefly-III `TransactionStore` request body.
- On success, returns HTTP 200 with the created transaction in `TransactionSingle` format.
- Validates required fields (`type`, `date`, `amount`, `description`); returns HTTP 422 with compatible error body on invalid input.
- Monetary `amount` fields must be accepted and stored as strings; rounding or truncation is not permitted.
- Account balances are updated atomically with the transaction creation.
- Requires valid bearer token.

---

### US-009 — Update Transaction
**Priority**: Must

As Maya (Household End User), I want to edit an existing transaction so that I can correct mistakes in my recorded transactions.

**Acceptance Criteria**:
- `PUT /api/v1/transactions/{id}` accepts the Firefly-III `TransactionUpdate` request body.
- On success, returns HTTP 200 with the updated transaction in `TransactionSingle` format.
- Account balances are recalculated correctly after the update.
- Returns HTTP 404 if not found or not owned by the authenticated user.
- Returns HTTP 422 on invalid input.
- Requires valid bearer token.

---

### US-010 — Delete Transaction
**Priority**: Must

As Maya (Household End User), I want to delete an erroneous transaction so that incorrect entries do not affect my account balance.

**Acceptance Criteria**:
- `DELETE /api/v1/transactions/{id}` returns HTTP 204 on success.
- Account balances are recalculated correctly after deletion.
- Returns HTTP 404 if not found or not owned by the authenticated user.
- Requires valid bearer token.

---

### US-011 — List Account Transactions
**Priority**: Must

As Maya (Household End User), I want to list transactions belonging to a specific account so that my client app can show per-account transaction history.

**Acceptance Criteria**:
- `GET /api/v1/accounts/{id}/transactions` returns HTTP 200 with a paginated list in `TransactionRead` format.
- Supports `?type=`, `?start=`, `?end=` filters.
- Returns HTTP 404 if the account does not exist or is not owned by the authenticated user.
- Requires valid bearer token.

---

## Epic: Budget Management

### US-012 — List Budgets
**Priority**: Should

As Maya (Household End User), I want to retrieve a list of budgets so that my client app can display my budget limits and spending status.

**Acceptance Criteria**:
- `GET /api/v1/budgets` returns HTTP 200 with a paginated list in Firefly-III `BudgetRead` list format.
- Each budget object includes: `id`, `name`, `active`, `auto_budget_type`, `auto_budget_amount`, `spent[]`, `budgeted[]`.
- Pagination follows Firefly-III convention.
- Requires valid bearer token.

---

### US-013 — Get Single Budget
**Priority**: Should

As Maya (Household End User), I want to retrieve a single budget's details so that my client app can show its current limit and spending.

**Acceptance Criteria**:
- `GET /api/v1/budgets/{id}` returns HTTP 200 in `BudgetSingle` format.
- Returns HTTP 404 if not found or not owned by the authenticated user.
- Requires valid bearer token.

---

### US-014 — Create Budget
**Priority**: Should

As Kenji (Self-Hosting Admin), I want to create a budget via the API so that spending limits can be defined for household categories.

**Acceptance Criteria**:
- `POST /api/v1/budgets` accepts the Firefly-III `BudgetStore` request body.
- On success, returns HTTP 200 in `BudgetSingle` format.
- Returns HTTP 422 on missing or invalid fields.
- Requires valid bearer token.

---

### US-015 — Update Budget
**Priority**: Should

As Kenji (Self-Hosting Admin), I want to update an existing budget via the API so that spending limits can be adjusted as household needs change.

**Acceptance Criteria**:
- `PUT /api/v1/budgets/{id}` accepts the Firefly-III `BudgetUpdate` request body.
- On success, returns HTTP 200 in `BudgetSingle` format.
- Returns HTTP 404 or HTTP 422 as appropriate.
- Requires valid bearer token.

---

### US-016 — Delete Budget
**Priority**: Could

As Kenji (Self-Hosting Admin), I want to delete an obsolete budget via the API so that the budget list stays clean and accurate.

**Acceptance Criteria**:
- `DELETE /api/v1/budgets/{id}` returns HTTP 204 on success.
- Returns HTTP 404 if not found or not owned by the authenticated user.
- Requires valid bearer token.

---

### US-017 — List Budget Limits
**Priority**: Should

As Maya (Household End User), I want to retrieve the spending limits set for a budget period so that my client app can show how much of a budget has been consumed.

**Acceptance Criteria**:
- `GET /api/v1/budgets/{id}/limits` returns HTTP 200 with a list in Firefly-III `BudgetLimitArray` format.
- Supports `?start=` and `?end=` date range parameters.
- Returns HTTP 404 if the budget does not exist or is not owned by the authenticated user.
- Requires valid bearer token.

---

## Epic: Common Metadata

### US-018 — List Currencies
**Priority**: Must

As Alex (Third-Party Client Developer), I want to retrieve the list of available currencies so that my client app can populate currency pickers correctly.

**Acceptance Criteria**:
- `GET /api/v1/currencies` returns HTTP 200 with a paginated list in Firefly-III `CurrencyArray` format.
- Each currency includes: `id`, `code`, `name`, `symbol`, `decimal_places`, `default`, `enabled`.
- Requires valid bearer token.

---

### US-019 — Get System Preferences / User Profile
**Priority**: Must

As Alex (Third-Party Client Developer), I want to retrieve the authenticated user's profile and default settings so that my client app can configure itself appropriately on first connect.

**Acceptance Criteria**:
- `GET /api/v1/about/user` returns HTTP 200 with Firefly-III `UserSingle` format.
- Response includes: `id`, `email`, `blocked`, `blocked_code`, `role`.
- Requires valid bearer token.

---

### US-020 — Get Server About Info
**Priority**: Must

As Alex (Third-Party Client Developer), I want to retrieve server version and capability metadata so that my client app can determine which API features are available.

**Acceptance Criteria**:
- `GET /api/v1/about` returns HTTP 200 with Firefly-III `SystemInfo` format.
- Response includes: `version`, `api_version`, `php_version` (or equivalent runtime info), `os`, `driver`.
- Requires valid bearer token.

---

## Epic: Authentication

### US-021 — Obtain OAuth2 / Personal Access Token
**Priority**: Must

As Kenji (Self-Hosting Admin), I want to issue a personal access token for a user so that client apps can authenticate against the API.

**Acceptance Criteria**:
- Token issuance endpoint or mechanism exists and is documented.
- Issued tokens can be used as bearer tokens on all secured endpoints.
- Tokens can be revoked without restarting the service.

---

### US-022 — Reject Unauthenticated Requests
**Priority**: Must

As Alex (Third-Party Client Developer), I want unauthenticated API calls to return a clear 401 error so that my client app can detect and handle authentication failures gracefully.

**Acceptance Criteria**:
- All protected endpoints return HTTP 401 with a Firefly-III compatible error body when called without a valid token.
- Response body contains a `message` field with a human-readable explanation.

---

## INVEST Compliance Summary

| Story | Independent | Negotiable | Valuable | Estimable | Small | Testable |
|-------|-------------|------------|----------|-----------|-------|----------|
| US-001 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-002 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-003 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-004 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-005 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-006 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-007 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-008 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-009 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-010 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-011 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-012 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-013 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-014 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-015 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-016 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-017 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-018 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-019 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-020 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-021 | Yes | Yes | Yes | Yes | Yes | Yes |
| US-022 | Yes | Yes | Yes | Yes | Yes | Yes |

## Persona to Story Coverage Map

| Persona | Stories |
|---------|---------|
| Maya (Household End User) | US-001, US-002, US-006, US-007, US-008, US-009, US-010, US-011, US-012, US-013, US-017 |
| Kenji (Self-Hosting Admin) | US-003, US-004, US-005, US-014, US-015, US-016, US-021 |
| Alex (Third-Party Client Developer) | US-018, US-019, US-020, US-022 |

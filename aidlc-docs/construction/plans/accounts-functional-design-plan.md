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

## Planning Checklist
- [ ] Analyze Firefly-III account schema compatibility (nested models)
- [ ] List all supported account types and validation rules per type
- [ ] Define the ownership enforcement mechanism within CRUD flows
- [ ] Formulate account cascade delete policies (handling of referenced transactions)
- [ ] Design pagination and filtered listing boundaries for accounts
- [ ] Draft Functional Design artifact templates for generation
- [ ] Request planning approval and resolve design preferences

## Planning Questions

Please fill all `[Answer]:` fields with your design preferences.

### Question 1: Supported Account Types

Which account types are initially supported in UOW-02 to preserve compatibility with Firefly-III adapters?

A) Custom subset: only "asset" (asset_default) and "expense" accounts
B) Core set: "asset", "expense", "revenue", and "liability" accounts
C) Full set: exact coverage of all Firefly-III account types (including cash, credit card, etc.)
D) Other (please describe)

[Answer]: B

### Question 2: Balance Tracking & Calculation Strategy

How should account balances be tracked/recalculated?

A) Calculated dynamically at runtime from transaction history (slower but strictly accurate)
B) Cached directly in the `accounts` table and atomically updated via transactions (faster reads, requires coordination)
C) Both: Store an `initial_balance` on the account, with active balances dynamically calculated by adding transaction sums
D) Other (please describe)

[Answer]: C

### Question 3: Soft vs Hard Deletion

What deletion strategy should apply when an account is deleted?

A) Strict hard delete from database (raises validation error if transactions exist)
B) Hard delete with cascade delete of all associated transactions (automatically destroys history)
C) Soft delete (`deleted_at` timestamp) to preserve financial ledger integrity even if hidden from UI
D) Other (please describe)

[Answer]: A

### Question 4: Default Currency Handling

How should currency associations be handled on Account creation?

A) Strictly require a currency ID/code on creation; fail if not provided
B) Fallback to a single global default currency (e.g. USD / JPY) if not defined
C) Context-derived default based on user preferences in their principal context
D) Other (please describe)

[Answer]: B

### Question 5: Firefly-III JSON Compatibility Level

How strictly does the response need to match the nested structural style of Firefly-III (specifically `{"data": {"type": "accounts", "id": "...", "attributes": {...}}}`)?

A) Strict: Response must serialize exactly into this nested envelope.
B) Partial: Provide customized DTO compatibility layer, but support flat payloads for custom internal integrations if needed.
C) Other (please describe)

[Answer]: A

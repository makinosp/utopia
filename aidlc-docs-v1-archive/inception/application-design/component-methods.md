# Component Methods

## Notes
- Method signatures are high-level and language-oriented for Rust implementation planning.
- Detailed business rules are deferred to Functional Design in CONSTRUCTION phase.

## Auth Component

### Trait AuthService
```rust
fn authenticate_bearer(token: &str) -> Result<Principal, AuthError>;
fn revoke_token(token_id: TokenId, principal: &Principal) -> Result<(), AuthError>;
```

## Accounts Component

### Trait AccountService
```rust
fn list_accounts(query: AccountListQuery, principal: &Principal) -> Result<Paginated<AccountView>, DomainError>;
fn get_account(id: AccountId, principal: &Principal) -> Result<AccountView, DomainError>;
fn create_account(input: CreateAccountInput, principal: &Principal) -> Result<AccountView, DomainError>;
fn update_account(id: AccountId, input: UpdateAccountInput, principal: &Principal) -> Result<AccountView, DomainError>;
fn delete_account(id: AccountId, principal: &Principal) -> Result<(), DomainError>;
```

## Transactions Component

### Trait TransactionService
```rust
fn list_transactions(query: TransactionListQuery, principal: &Principal) -> Result<Paginated<TransactionGroupView>, DomainError>;
fn get_transaction(id: TransactionId, principal: &Principal) -> Result<TransactionGroupView, DomainError>;
fn create_transaction(input: CreateTransactionInput, principal: &Principal) -> Result<TransactionGroupView, DomainError>;
fn update_transaction(id: TransactionId, input: UpdateTransactionInput, principal: &Principal) -> Result<TransactionGroupView, DomainError>;
fn delete_transaction(id: TransactionId, principal: &Principal) -> Result<(), DomainError>;
fn list_account_transactions(account_id: AccountId, query: TransactionListQuery, principal: &Principal) -> Result<Paginated<TransactionGroupView>, DomainError>;
```

## Budgets Component

### Trait BudgetService
```rust
fn list_budgets(query: BudgetListQuery, principal: &Principal) -> Result<Paginated<BudgetView>, DomainError>;
fn get_budget(id: BudgetId, principal: &Principal) -> Result<BudgetView, DomainError>;
fn create_budget(input: CreateBudgetInput, principal: &Principal) -> Result<BudgetView, DomainError>;
fn update_budget(id: BudgetId, input: UpdateBudgetInput, principal: &Principal) -> Result<BudgetView, DomainError>;
fn delete_budget(id: BudgetId, principal: &Principal) -> Result<(), DomainError>;
fn list_budget_limits(id: BudgetId, query: DateRangeQuery, principal: &Principal) -> Result<Vec<BudgetLimitView>, DomainError>;
```

## Metadata Component

### Trait MetadataService
```rust
fn list_currencies(query: PaginationQuery, principal: &Principal) -> Result<Paginated<CurrencyView>, DomainError>;
fn get_about_user(principal: &Principal) -> Result<UserView, DomainError>;
fn get_about_system(principal: &Principal) -> Result<SystemInfoView, DomainError>;
```

## Compatibility Component

### Trait CompatibilityMapper
```rust
fn map_account_to_firefly(view: AccountView) -> FireflyAccountResource;
fn map_transaction_to_firefly(view: TransactionGroupView) -> FireflyTransactionResource;
fn map_budget_to_firefly(view: BudgetView) -> FireflyBudgetResource;
fn map_pagination(meta: PaginationMeta) -> FireflyPaginationMeta;
```

## Error Mapping Component

### Trait ApiErrorMapper
```rust
fn map_domain_error(err: DomainError) -> FireflyErrorResponse;
fn map_validation_error(err: ValidationError) -> FireflyErrorResponse;
fn map_auth_error(err: AuthError) -> FireflyErrorResponse;
```

## Persistence Component

- Request-scoped transactions are managed explicitly by application services using `sqlx::PgPool::begin()` and `sqlx::Transaction`.
- Repository write methods receive `&mut sqlx::Transaction<'_, sqlx::Postgres>`; read methods may use either the pool or a transaction via `sqlx::Executor`.

### Repository Traits
```rust
trait AccountRepository { /* account reads and writes */ }
trait TransactionRepository { /* transaction reads and writes */ }
trait BudgetRepository { /* budget reads and writes */ }
trait MetadataRepository { /* metadata reads */ }
```

## API Handler Component

### Handler Contract Pattern
```rust
async fn handle_<operation>(request: HttpRequest, deps: HandlerDeps) -> HttpResponse;
```

Handler responsibilities:
- Parse and validate transport-level inputs.
- Resolve principal using AuthService.
- Call domain service method.
- Map domain views and errors through compatibility and error mapping components.

# Application Design Plan

## Objective
Define high-level application architecture for a Rust-based household finance API with partial Firefly-III compatibility.

## Plan Checklist
- [x] Confirm design assumptions from requirements and stories
- [x] Finalize component boundaries
- [x] Define component interfaces and method signatures
- [x] Define service layer and orchestration rules
- [x] Define component dependency and communication patterns
- [x] Generate `aidlc-docs/inception/application-design/components.md`
- [x] Generate `aidlc-docs/inception/application-design/component-methods.md`
- [x] Generate `aidlc-docs/inception/application-design/services.md`
- [x] Generate `aidlc-docs/inception/application-design/component-dependency.md`
- [x] Generate `aidlc-docs/inception/application-design/application-design.md`
- [x] Validate design completeness and consistency
- [x] Prepare review summary for approval gate

## Design Questions

Please fill all `[Answer]:` fields.

## Question 1
What should be the primary component decomposition strategy?

A) Domain-first (Accounts, Transactions, Budgets, Metadata, Auth)
B) Layer-first (Controllers, Services, Repositories, Models)
C) Hybrid (Domain modules with internal layered structure)
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 2
How should compatibility logic be organized?

A) Keep Firefly-compatible DTO mapping inside each domain component
B) Centralize compatibility adapters in a shared compatibility component
C) Hybrid: shared primitives + domain-local adapters
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 3
What service orchestration style should be used for write operations (create/update/delete)?

A) Thin service layer, most logic inside domain components
B) Rich application service layer orchestrating validation, authorization, persistence
C) Command-handler style per operation
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 4
How should transaction consistency boundaries be defined?

A) One DB transaction per API request that mutates state
B) Fine-grained transactions per repository method
C) Mixed: default request-level, with documented exceptions
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 5
Where should authorization checks be enforced primarily?

A) API handler level only
B) Service level only
C) Both API handler and service level (defense-in-depth)
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 6
How should monetary values be handled across components?

A) Decimal domain type end-to-end, string only at API boundary
B) String end-to-end for strict compatibility
C) Hybrid: decimal in domain/service, compatibility string in DTO layer
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 7
What communication pattern should components use internally?

A) Direct synchronous calls between components
B) Event-driven internal messages for state changes
C) Hybrid: synchronous for reads, event-driven for selected writes
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 8
How should error mapping be standardized?

A) Central error mapper translating domain errors to Firefly-compatible responses
B) Each component maps its own errors independently
C) Hybrid: central core mapper + component-specific extensions
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 9
How should component interfaces be exposed for testing and maintainability?

A) Trait-based interfaces for all service and repository boundaries
B) Concrete types first, traits only where needed
C) Traits for repositories only, concrete services
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 10
What infrastructure abstraction level should Application Design assume now?

A) Database-agnostic abstraction first; vendor-specific details later
B) PostgreSQL-first design from the beginning
C) SQLite-first for local, upgrade path documented for PostgreSQL
X) Other (please describe after [Answer]: tag below)

[Answer]: B

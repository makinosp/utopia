# Application Design

## Summary
This document consolidates the Application Design artifacts for the Rust-based, partially Firefly-compatible household finance API.

Design decisions are based on approved answers in the application design plan:
- Decomposition: Hybrid (domain modules with internal layers)
- Compatibility strategy: Shared primitives plus domain-local adapters
- Service orchestration: Rich application service layer
- Transaction boundary: One DB transaction per mutating request
- Authorization: Defense in depth at handler and service levels
- Monetary handling: Decimal internal model with DTO string conversion
- Internal communication: Direct synchronous calls
- Error mapping: Central core mapper with domain extension points
- Interface style: Repository traits with concrete services
- Infrastructure assumption: PostgreSQL-first

## Artifact Index
- Components: see components.md
- Component methods: see component-methods.md
- Services: see services.md
- Component dependencies: see component-dependency.md

## High-Level Structure
- Transport layer: API Handler Component.
- Security layer: Auth Component.
- Domain modules: Accounts, Transactions, Budgets, Metadata.
- Cross-cutting modules: Compatibility Component, Error Mapping Component.
- Persistence layer: Repository traits plus PostgreSQL adapters and transaction manager.

## Design Completeness Check
- Component boundaries are defined.
- Service responsibilities and orchestration are defined.
- Method signatures are defined at planning level.
- Dependency and communication rules are defined.
- Firefly compatibility strategy is defined.
- Security and consistency baseline is captured.

## Deferred to Functional Design
- Detailed validation rules and invariants.
- Exact repository query contracts and indexing strategy.
- Detailed command/query models per endpoint.
- Complete error code matrix per operation.
- Property-based test case specifications per unit.

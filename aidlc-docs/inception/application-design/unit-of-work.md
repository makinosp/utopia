# Unit of Work Definition

## Objective
Decompose the approved API design into implementable units with clear ownership, strict boundaries, and delivery sequence for the CONSTRUCTION phase.

## Decomposition Decisions (Approved)
- Grouping strategy: Hybrid domain-first with one integration/core unit.
- Unit count target: 5+ small units.
- Cross-cutting placement: Shared core primitives with domain-owned adapters.
- Dependency style: Strict acyclic dependencies.
- Story assignment: One primary unit per story, secondary units tracked as dependencies.
- Delivery order: Foundation first, then domain units.
- Ownership model: Virtual ownership per unit.
- Primary boundary goal: Domain correctness and API contract stability.
- Greenfield organization: Single package now with explicit migration path to multi-package.
- Boundary strictness: Moderate (allow constrained adjustments during Functional Design).

## Unit Catalog

| Unit ID | Unit Name | Primary Scope | Primary Stories | Owner Model | Delivery Order |
|---|---|---|---|---|---|
| UOW-01 | Core Foundation | Auth, API compatibility mapping, API error mapping, request transaction boundary, shared contracts | US-021, US-022 | Virtual Owner: Core | 1 |
| UOW-02 | Accounts Module | Account CRUD, account listing and filtering, account ownership rules | US-001, US-002, US-003, US-004, US-005 | Virtual Owner: Accounts | 2 |
| UOW-03 | Transactions Module | Transaction CRUD, account transaction listing, atomic balance-affecting flows | US-006, US-007, US-008, US-009, US-010, US-011 | Virtual Owner: Transactions | 3 |
| UOW-04 | Budgets Module | Budget CRUD and budget limits by period | US-012, US-013, US-014, US-015, US-016, US-017 | Virtual Owner: Budgets | 4 |
| UOW-05 | Metadata Module | Currencies and about endpoints | US-018, US-019, US-020 | Virtual Owner: Metadata | 5 |

## Unit Responsibilities

### UOW-01 Core Foundation
- Provide bearer authentication and principal resolution.
- Provide shared Firefly compatibility primitives for pagination and envelope consistency.
- Provide centralized error-to-Firefly payload mapping.
- Provide request-scoped transaction manager abstractions used by mutating domain operations.
- Expose shared interfaces consumed by domain units without introducing reverse dependencies.

### UOW-02 Accounts Module
- Implement account list/single/create/update/delete operations.
- Enforce ownership checks via principal context.
- Publish account domain views for API mapping.

### UOW-03 Transactions Module
- Implement transaction list/single/create/update/delete operations.
- Implement per-account transaction listing.
- Guarantee atomic balance updates and recalculation behavior for write operations.

### UOW-04 Budgets Module
- Implement budget list/single/create/update/delete operations.
- Implement budget limits retrieval with date range constraints.
- Expose budget views compatible with Firefly response contracts.

### UOW-05 Metadata Module
- Implement currencies list endpoint behavior.
- Implement about and about user endpoint behavior.
- Keep metadata flows read-oriented and lightweight.

## Boundary Rules
- No bidirectional unit dependencies.
- Domain units must not depend on each other directly for mutable operations.
- Domain units consume UOW-01 shared interfaces only.
- Firefly DTO shapes are produced through compatibility adapters rather than leaking transport contracts into domain logic.
- Monetary precision rules stay domain-safe internally and string-compatible at API boundaries.

## Greenfield Code Organization Strategy
Initial strategy is a single service package with explicit migration seams.

Proposed initial layout:

```text
src/
  core/
    auth/
    compatibility/
    error_mapping/
    persistence/
  modules/
    accounts/
    transactions/
    budgets/
    metadata/
  api/
    handlers/
```

Migration path to multi-package:
- Keep interfaces in each unit boundary explicit.
- Avoid cross-unit internal imports outside approved interfaces.
- Promote `core` and each module to dedicated packages when scaling criteria are met.

## Readiness for CONSTRUCTION
- Unit boundaries are explicit and testable.
- Ownership and delivery order are defined.
- Cross-cutting concerns are centralized with domain adapters.
- All approved user stories are mapped to a primary unit.

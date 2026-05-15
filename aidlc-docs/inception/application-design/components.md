# Components

## Overview
Primary architecture style: hybrid component decomposition (domain modules with internal layered structure).

## Component Catalog

## 1. Auth Component
- Purpose: Authenticate API callers and produce authenticated principal context.
- Responsibilities:
  - Validate bearer tokens.
  - Resolve authenticated user identity.
  - Provide authorization guard primitives for service layer checks.
- External interface:
  - Token validation entry points used by API handlers.
  - Principal context provider consumed by services.

## 2. Accounts Component
- Purpose: Manage account lifecycle and account balance views.
- Responsibilities:
  - CRUD operations for accounts.
  - Account ownership and access scope enforcement at service boundary.
  - Balance representation for API responses.
- External interface:
  - Account query and mutation operations.

## 3. Transactions Component
- Purpose: Manage transaction recording and balance-affecting operations.
- Responsibilities:
  - CRUD operations for transactions.
  - Atomic balance updates for create/update/delete flows.
  - Date and type filtering for account and global transaction listing.
- External interface:
  - Transaction query and mutation operations.

## 4. Budgets Component
- Purpose: Manage budgets and budget limits over time windows.
- Responsibilities:
  - CRUD operations for budgets.
  - Budget limit listing by date range.
  - Budget usage retrieval support for API read models.
- External interface:
  - Budget query and mutation operations.

## 5. Metadata Component
- Purpose: Provide common metadata endpoints and server/user informational APIs.
- Responsibilities:
  - Currencies listing.
  - About endpoint payloads (system info and authenticated user profile).
- External interface:
  - Metadata retrieval operations.

## 6. Compatibility Component
- Purpose: Provide Firefly-III compatibility mapping primitives.
- Responsibilities:
  - Shared DTO conventions and pagination envelope helpers.
  - Domain-local adapter contracts for response and error compatibility.
  - Contract version and compatibility scope registry for supported endpoints.
- External interface:
  - Mapping primitives used by each domain component.

## 7. Error Mapping Component
- Purpose: Standardize domain-to-API error translation.
- Responsibilities:
  - Map domain and validation errors to Firefly-compatible error payloads.
  - Provide centralized mapping policies with extension hooks per domain.
- External interface:
  - Error translation service used by handlers and service middleware.

## 8. Persistence Component
- Purpose: Encapsulate database access and transactional boundaries.
- Responsibilities:
  - Repository implementations and transaction manager.
  - PostgreSQL-first schema access patterns.
  - Unit-of-work style request transaction helpers for mutating requests.
- External interface:
  - Repository traits and transaction manager interfaces.

## 9. API Handler Component
- Purpose: HTTP transport adapter and endpoint wiring.
- Responsibilities:
  - Parse request inputs and invoke application services.
  - Execute preliminary auth checks and input validation.
  - Delegate response/error serialization to compatibility and error mapping components.
- External interface:
  - Public HTTP routes under /api/v1.

## Shared Design Rules
- Domain ownership: Accounts, Transactions, Budgets, and Metadata remain domain-owned modules.
- Layering inside each domain: handler-facing contract -> service interface -> repository interface.
- Compatibility strategy: shared primitives plus domain-local adapters.
- Security strategy: defense-in-depth authorization at handler and service layers.
- Monetary strategy: decimal domain values internally; string representation at DTO boundaries.

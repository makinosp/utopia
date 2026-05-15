# Services

## Service Layer Strategy
Chosen orchestration style: rich application service layer for write operations.

## Service Definitions

## 1. Authentication Service
- Coordinates token validation and principal resolution.
- Provides revocation-related operations.
- Exposed to API handlers as first security gate.

## 2. Account Application Service
- Orchestrates account command and query flows.
- Enforces account ownership and authorization checks.
- Coordinates repository interaction and output view construction.

## 3. Transaction Application Service
- Orchestrates transaction command and query flows.
- Coordinates atomic balance adjustments with transaction writes.
- Applies request-scoped transaction boundary for all mutations.

## 4. Budget Application Service
- Orchestrates budget and budget-limit use cases.
- Enforces ownership and date-range consistency checks.
- Coordinates repository access and response assembly.

## 5. Metadata Application Service
- Handles currencies and about endpoints.
- Provides lightweight read-oriented orchestration.

## 6. Compatibility Mapping Service
- Provides shared mapping utilities plus domain adapter hooks.
- Ensures Firefly-compatible response envelope consistency.

## 7. API Error Mapping Service
- Centralizes error conversion to Firefly-compatible error payload schema.
- Supports domain-specific mapping extensions while preserving standard shape.

## Orchestration Patterns

## Write Flow Pattern (create/update/delete)
1. Handler validates transport-level input.
2. Auth service resolves principal.
3. Service performs policy and domain-level authorization.
4. Service invokes transaction manager for request-scoped DB transaction.
5. Service performs repository operations.
6. Service returns domain view.
7. Compatibility mapping service builds Firefly-compatible response.

## Read Flow Pattern
1. Handler validates query/path parameters.
2. Auth service resolves principal.
3. Service performs ownership scope checks.
4. Service executes read repository operations.
5. Compatibility mapping service returns Firefly-compatible list/single payload.

## Security and Consistency Policies in Services
- Defense in depth: auth checks at handler and service boundaries.
- Mutating requests: one DB transaction per request.
- Errors: centralized mapping to consistent API error payload.
- Monetary handling: decimal in domain/service, string at DTO boundary.

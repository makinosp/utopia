# Functional Design Plan - Core Foundation (UOW-01)

## Objective
Define detailed business logic for authentication, API compatibility mapping, error handling, transaction boundaries, and shared contracts that all domain units depend on.

## Unit Context
- **Unit Name**: Core Foundation (UOW-01)
- **Primary Stories**: US-021 (Obtain OAuth2 / Personal Access Token), US-022 (Reject Unauthenticated Requests)
- **Key Responsibilities**:
  - Bearer token authentication and principal resolution
  - Firefly-III compatibility primitives (pagination, response envelopes, DTOs)
  - Centralized error-to-Firefly payload mapping
  - Request-scoped transaction manager abstractions
  - Shared data access patterns and contracts

## Planning Checklist
- [x] Analyze authentication requirements and token lifecycle
- [x] Define principal representation (user context for authorization scopes)
- [x] Analyze Firefly compatibility requirements (pagination, response structure, error format)
- [x] Define centralized error categorization and mapping rules
- [x] Analyze transaction boundary strategy and isolation requirements
- [x] Define shared data contract primitives consumed by domain units
- [x] Generate functional design artifacts
- [x] Request planning approval to proceed to artifact generation

## Planning Questions

Please fill all `[Answer]:` fields with your design preferences.

### Question 1: Token Lifecycle and Issuance

How should personal access tokens be issued and managed?

A) Admin-issued tokens with fixed expiration date (e.g., 90 days)
B) Long-lived tokens with no expiration; revocation only via explicit admin action
C) Hybrid: initial issue with long lifetime + optional manual extension/refresh endpoints
D) Short-lived tokens with refresh token exchange flow (OAuth2 pattern)
E) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 2: Principal Context Representation

What information should be captured in the authenticated principal context?

A) Minimal: user_id only
B) Lightweight: user_id + email
C) Comprehensive: user_id + email + role + account_type + account_id (for self-hosted multi-tenant scenarios)
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 3: Token Validation Scenario Handling

How should the system respond to different token validation failures?

A) Return HTTP 401 for all invalid token scenarios (expired, revoked, malformed, missing)
B) Distinguish scenarios: 401 for missing/malformed, 403 for revoked/expired
C) Include detailed reason in error response body (e.g., "token_expired", "token_revoked")
D) Other (please describe after [Answer]: tag below)

[Answer]: C

### Question 4: Firefly Error Response Structure

Which error payload structure should all errors conform to?

A) Simple: `{ message: "Human-readable error" }`
B) Standard Firefly: `{ message: "...", errors: { field: ["error message"] } }`
C) Rich: `{ message: "...", errors: {...}, code: "ERROR_CODE", timestamp: "iso8601" }`
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 5: Pagination Metadata Structure

What pagination information should be included in list responses?

A) Minimal: total, per_page, current_page
B) Firefly standard: total, count, per_page, current_page, total_pages
C) Expanded: includes has_next, has_previous, first_page_url, last_page_url
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 6: Firefly DTO Conventions

How strictly should DTOs conform to Firefly-III structures?

A) Exact field-by-field match to Firefly-III schema (strict compatibility)
B) Compatible but allow internal Utopia extensions for new fields (forward compatibility)
C) Subset match: include only essential Firefly fields, omit less-used ones
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 7: Transaction Boundary Granularity

What should define a transaction boundary in this system?

A) One transaction per HTTP request (all domain writes within one tx)
B) Fine-grained per-operation transactions (each domain mutation owns its tx)
C) Explicit tx control: handlers request tx, services can nest/compose
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 8: Decimal Precision in Monetary Amounts

How should monetary amounts be handled at the domain level?

A) Decimal type with fixed scale (e.g., 2 decimal places always)
B) Decimal type with variable scale preserved from input
C) Internal string representation to prevent precision loss
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 9: Shared Data Access Patterns

Which data access abstraction should be the foundation for all domain units?

A) Repository trait pattern: each domain owns repository interfaces
B) Query builder pattern: shared query composition with domain-specific filters
C) Hybrid: shared base repository traits + domain-specific implementations
D) Other (please describe after [Answer]: tag below)

[Answer]: C

### Question 10: Cross-Cutting Security Policy

How should authorization scope be enforced at the Core Foundation level?

A) Principal context only: services decide ownership rules
B) Pre-filter at auth layer: only return user-owned records in queries
C) Hybrid: auth provides context, services add domain-specific rules, handlers validate scope
D) Other (please describe after [Answer]: tag below)

[Answer]: B

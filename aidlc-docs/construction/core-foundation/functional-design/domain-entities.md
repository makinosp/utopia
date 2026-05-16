# Domain Entities - Core Foundation (UOW-01)

## Design Decisions Applied
- Principal: user_id + email (Q2: B)
- Token: long-lived, admin-revocable only (Q1: B)
- Pagination: Firefly standard fields (Q5: B)
- Error payload: Firefly standard structure (Q4: B)
- Monetary precision: variable-scale decimal (Q8: B)
- DTO compatibility: compatible + allow Utopia extensions (Q6: B)

---

## Entity: Token

Represents a personal access token issued to a user for authenticating API requests.

| Field | Type | Description |
|---|---|---|
| id | TokenId (UUID) | Unique token identifier |
| user_id | UserId (UUID) | Owner of the token |
| token_hash | String | Bcrypt/SHA-256 hash of the raw token value; raw value is never stored |
| label | String | Human-readable label for identifying the token |
| status | TokenStatus | Current state of the token |
| created_at | DateTime (UTC) | Token creation timestamp |
| last_used_at | Option\<DateTime\> | Last successful authentication timestamp |

### TokenStatus Enum

| Variant | Description |
|---|---|
| Active | Token is valid for authentication |
| Revoked | Token has been revoked by admin; cannot be used |

### Invariants
- `token_hash` must never be empty.
- A revoked token must remain in the store for audit purposes; deletion is not permitted.
- Only a user with Admin role may revoke tokens. (Role is resolved from the users table, not stored in the token.)
- There is no expiry field; tokens live indefinitely unless revoked.

---

## Entity: User

Represents an authenticated user in the system.

| Field | Type | Description |
|---|---|---|
| id | UserId (UUID) | Unique user identifier |
| email | String | Unique email address |
| password_hash | String | Hashed user password; never stored in plain text |
| blocked | bool | Whether the user account is blocked |
| blocked_code | Option\<String\> | Reason code if the user is blocked |
| role | UserRole | Role determining administrative capabilities |
| created_at | DateTime (UTC) | Account creation timestamp |
| updated_at | DateTime (UTC) | Last modification timestamp |

### UserRole Enum

| Variant | Description |
|---|---|
| Owner | Full administrative access; can manage tokens and settings |
| ReadOnly | Read-only API access |

---

## Value Object: Principal

Represents the authenticated user context propagated through all request-scoped operations. Produced by `AuthService` after successful token validation.

| Field | Type | Description |
|---|---|---|
| user_id | UserId (UUID) | Resolved owner identifier |
| email | String | Resolved email of the authenticated user |

### Invariants
- `Principal` is always constructed after successful token validation; no unauthenticated principal exists.
- `user_id` is the primary key used for ownership-scoped queries and mutations.

---

## Value Object: Decimal Amount

Represents a monetary amount at the domain layer with variable-scale precision.

| Field | Type | Description |
|---|---|---|
| value | rust_decimal::Decimal | Arbitrary-precision decimal value |

### Invariants
- Input scale is preserved from the deserialized string.
- No rounding is applied at the domain layer.
- Conversion to/from API string representation (`"10.50"`) happens only at DTO boundaries.
- Negative values are permitted for debit-side representations.

---

## Value Object: PaginationMeta

Pagination metadata returned in all list responses, conforming to the Firefly-III pagination shape.

| Field | Type | Description |
|---|---|---|
| total | u64 | Total number of records matching the query |
| count | u64 | Number of records returned in this page |
| per_page | u32 | Page size requested |
| current_page | u32 | Current page number (1-indexed) |
| total_pages | u32 | Total number of pages |

---

## Value Object: FireflyErrorResponse

Standardized error payload returned to API consumers, conforming to the Firefly-III error schema.

| Field | Type | Description |
|---|---|---|
| message | String | Human-readable summary of the error; includes reason code for auth errors |
| errors | Map\<String, Vec\<String\>\> | Field-level validation errors; empty for non-validation errors |

### Auth Error Message Convention
For authentication failures (Q3: C), the `message` field carries a machine-readable reason identifier in addition to a human-readable description. Format: `"<reason_code>: <human description>"`.

Defined reason codes:

| Reason Code | Scenario |
|---|---|
| `unauthenticated` | Bearer token is missing from the request |
| `token_malformed` | Token value fails format validation |
| `token_revoked` | Token exists but has been revoked |
| `token_not_found` | Token value not found in the store |
| `user_blocked` | Token is valid but the owning user account is blocked |

All authentication failures return **HTTP 401**.

---

## Value Object: FireflyListEnvelope\<T\>

Generic wrapper for all list responses, conforming to Firefly-III list response envelope.

| Field | Type | Description |
|---|---|---|
| data | Vec\<T\> | Page of resource objects |
| meta | PaginationMeta | Pagination metadata |

---

## Value Object: FireflySingleEnvelope\<T\>

Generic wrapper for single-resource responses.

| Field | Type | Description |
|---|---|---|
| data | T | Single resource object |

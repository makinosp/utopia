# Business Rules - Core Foundation (UOW-01)

## Design Decisions Applied
- Token: long-lived, admin-revocable (Q1: B)
- Auth error handling: 401 for all, reason code in body (Q3: C)
- Error format: Firefly standard (Q4: B)
- Transaction boundary: one per HTTP request (Q7: A)
- Authorization scope: pre-filter at auth layer (Q10: B)
- DTO compatibility: compatible + Utopia extensions (Q6: B)

---

## Authentication Rules

### BR-AUTH-001: Bearer Token Required
All API endpoints except token issuance are protected.
- If the `Authorization: Bearer <token>` header is absent, return HTTP 401 with reason code `unauthenticated`.
- If the token value fails format validation (not a valid UUID or opaque token format), return HTTP 401 with reason code `token_malformed`.

### BR-AUTH-002: Token Lookup and Status Check
On every authenticated request:
1. Hash the presented token value using the same algorithm used at issuance.
2. Look up the token by hash in the token store.
3. If no matching token found, return HTTP 401 with reason code `token_not_found`.
4. If the token status is `Revoked`, return HTTP 401 with reason code `token_revoked`.
5. Load the owning user record.
6. If the user is blocked (`blocked = true`), return HTTP 401 with reason code `user_blocked`.
7. If all checks pass, construct and return a `Principal` value object.

### BR-AUTH-003: Token Issuance
- Tokens are generated as cryptographically random opaque values (minimum 32 bytes entropy).
- Only the hash is stored; the raw token is returned once at issuance and cannot be retrieved again.
- Tokens have no expiration timestamp.
- A user may hold multiple active tokens.

### BR-AUTH-004: Token Revocation
- Only admin-level users (role = Owner) may revoke any token.
- A user with any role may revoke their own tokens.
- Revoked tokens must remain in the store; they must not be deleted.
- Last-used-at timestamp must be updated on each successful authentication.

---

## Error Mapping Rules

### BR-ERR-001: Firefly Standard Error Format
All error responses must conform to the `FireflyErrorResponse` structure:
```
{ "message": "<string>", "errors": { "<field>": ["<message>"] } }
```
- `message`: human-readable summary plus reason code for auth errors.
- `errors`: populated only for validation failures; empty map `{}` for non-validation errors.

### BR-ERR-002: HTTP Status Code Assignment

| Error Category | HTTP Status |
|---|---|
| Authentication failure (all cases) | 401 |
| Resource not found | 404 |
| Input validation failure | 422 |
| Unexpected server error | 500 |

Note: 403 is not used; all access denial due to authentication maps to 401.

**Detailed Specification:**
For a comprehensive error handling specification aligned with Firefly III conventions, see [firefly-error-contract.md](firefly-error-contract.md). This document includes:
- Complete error scenario matrix (9 auth scenarios, bootstrap scenarios, domain scenarios)
- Content-Type/Accept negotiation rules
- Message text standards and reason code formats
- Test verification procedures

### BR-ERR-003: Validation Error Format
For HTTP 422 responses, `errors` must map field names to arrays of error messages.
Example:
```json
{
  "message": "The given data was invalid.",
  "errors": {
    "name": ["The name field is required."],
    "type": ["The selected type is invalid."]
  }
}
```

### BR-ERR-004: Auth Error Message Format
Auth error messages must embed a reason code prefix:
```
"<reason_code>: <human-readable description>"
```
Example: `"token_revoked: The provided token has been revoked."`

---

## Pagination Rules

### BR-PAG-001: Pagination Parameters
List endpoints must accept `page` (default: 1) and `limit` (default: 50, max: 100) query parameters.

### BR-PAG-002: Pagination Metadata
All list responses must include a `meta.pagination` object with:
- `total`: total record count matching the query filters
- `count`: number of records in the current page
- `per_page`: requested page size
- `current_page`: requested page number
- `total_pages`: ceiling of total / per_page

### BR-PAG-003: Empty Result Sets
If no records match, return HTTP 200 with an empty `data` array and valid pagination metadata with `total = 0`.

---

## Transaction Boundary Rules

### BR-TX-001: One Transaction Per Mutating Request
All mutating HTTP requests (POST, PUT, DELETE) must execute within a single database transaction.
- The transaction begins before the first repository write operation.
- If any write fails, the entire transaction must be rolled back.
- Read-only requests (GET) do not require a wrapping transaction.

### BR-TX-002: Transaction Isolation
Transactions must use the database default isolation level (Read Committed for PostgreSQL).
Serializable isolation is not required for initial scope.

### BR-TX-003: Transaction Failure Handling
If the transaction cannot be committed (e.g., constraint violation, connection loss), the error must be mapped to an appropriate `FireflyErrorResponse` and returned as HTTP 500 unless the cause is a known domain constraint (in which case the most specific status is used).

---

## Authorization Scope Rules

### BR-AUTHZ-001: User-Scoped Data Access
All repository queries for user-owned resources must include a `user_id = principal.user_id` filter condition.
- This filter is applied at the repository layer, not in application service logic.
- Domain services must pass the `principal` to repository methods; repository implementations are responsible for applying the ownership filter.
- No cross-user data leakage is permitted.

### BR-AUTHZ-002: 404 for Unauthorized Resources
If a resource is not found because it belongs to a different user, return HTTP 404.
HTTP 403 must not be used for ownership failures, as it would reveal the existence of another user's resource.

---

## DTO Compatibility Rules

### BR-DTO-001: Firefly Field Compatibility
All response fields that correspond to Firefly-III schema fields must use identical field names, types, and value conventions.

### BR-DTO-002: Utopia Extension Fields
Additional Utopia-specific fields may be added to responses. Extension fields must:
- Not collide with any Firefly-III field name.
- Be additive only (no modification to existing Firefly fields).
- Be clearly documented in API documentation.

### BR-DTO-003: Monetary String Conversion
The `amount` field in all DTOs must be serialized as a JSON string.
Example: `"amount": "123.45"` not `"amount": 123.45`.
Deserialization must accept both string and numeric forms for forward compatibility, but must never truncate precision.

---

## Data Integrity Rules

### BR-INT-001: Timestamps
All timestamps must be stored and returned in UTC ISO 8601 format: `"YYYY-MM-DDTHH:MM:SS.sssZ"`.

### BR-INT-002: UUID Identifiers
All resource identifiers use UUID v4 format. Sequential integer IDs are not exposed in the API.

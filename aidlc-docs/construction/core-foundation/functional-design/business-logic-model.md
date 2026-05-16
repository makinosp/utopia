# Business Logic Model - Core Foundation (UOW-01)

## Overview
This document describes the business workflows for Core Foundation. These are technology-agnostic logic flows; implementation details (specific Rust types, database drivers) are deferred to Code Generation.

---

## Flow 1: Request Authentication (Used by All Protected Endpoints)

**Trigger**: Incoming HTTP request to any protected endpoint.

**Input**: Raw `Authorization` header value.

**Steps**:
1. Extract bearer token string from `Authorization: Bearer <token>` header.
   - If header is missing or the format is not `Bearer <value>`, produce auth failure: `unauthenticated`.
2. Validate token format (must match expected opaque token format).
   - If invalid format, produce auth failure: `token_malformed`.
3. Hash the token value using the stored hashing algorithm.
4. Look up token record by hash in the token store.
   - If not found, produce auth failure: `token_not_found`.
5. Check token status.
   - If `Revoked`, produce auth failure: `token_revoked`.
6. Load the owning user record by `token.user_id`.
7. Check user is not blocked.
   - If `blocked = true`, produce auth failure: `user_blocked`.
8. Update `last_used_at` on the token record (fire-and-forget; does not block response).
9. Construct `Principal { user_id, email }`.
10. Return `Principal` to caller.

**Auth Failure Output**:
- HTTP 401
- `FireflyErrorResponse { message: "<reason_code>: <description>", errors: {} }`

**Success Output**: `Principal` value object.

---

## Flow 2: Token Issuance

**Trigger**: Authenticated request to token issuance endpoint (US-021).

**Input**: Token label from request body; `Principal` of the requesting user.

**Steps**:
1. Generate a cryptographically random raw token value (minimum 32 bytes entropy, encoded as URL-safe base64).
2. Compute the hash of the raw token.
3. Persist a `Token` record with status `Active`, `user_id`, `label`, `token_hash`, and `created_at = now()`.
4. Return the raw token value once in the response. Raw value is not persisted and cannot be retrieved again.

**Output**:
```
{
  "token": "<raw_token_value>",
  "id": "<token_id>",
  "label": "<label>",
  "created_at": "<iso8601>"
}
```

---

## Flow 3: Token Revocation

**Trigger**: Authenticated request to token revocation endpoint.

**Input**: Token ID to revoke; `Principal` of the requesting user.

**Steps**:
1. Look up the token record by ID.
   - If not found, produce 404 error.
2. Verify authorization: the requesting user must be the token owner, OR must have `Owner` role.
   - If neither condition holds, produce 404 error (do not reveal existence of other users' tokens).
3. Set `token.status = Revoked`.
4. Persist the update.
5. Return HTTP 204 No Content.

---

## Flow 4: Error Mapping

**Trigger**: An error occurs anywhere in the request pipeline.

**Input**: Domain error, validation error, or unexpected error.

**Logic**:

```
match error type:
  AuthError =>
    HTTP 401, FireflyErrorResponse { message: "<reason_code>: ...", errors: {} }
  NotFoundError =>
    HTTP 404, FireflyErrorResponse { message: "Not found.", errors: {} }
  ValidationError(fields) =>
    HTTP 422, FireflyErrorResponse { message: "The given data was invalid.", errors: { field: [msg, ...] } }
  PersistenceError | UnexpectedError =>
    HTTP 500, FireflyErrorResponse { message: "An unexpected error occurred.", errors: {} }
```

**Rule**: The `errors` map is populated only for validation errors. All other error types produce an empty `errors: {}` map.

---

## Flow 5: Pagination Assembly

**Trigger**: Any list query result returned by a domain repository.

**Input**: Raw list query result containing total count and page of records.

**Steps**:
1. Receive: `total_records`, `page_records`, `current_page`, `per_page`.
2. Compute `total_pages = ceil(total_records / per_page)`.
3. Compute `count = len(page_records)`.
4. Construct `PaginationMeta { total, count, per_page, current_page, total_pages }`.
5. Wrap result: `FireflyListEnvelope { data: page_records, meta: { pagination: PaginationMeta } }`.

---

## Flow 6: Mutating Request Transaction Boundary

**Trigger**: Any service method that performs one or more write operations.

**Steps**:
1. Begin a database transaction.
2. Execute all repository write operations within the transaction context.
3. If all writes succeed, commit the transaction.
4. If any write fails:
   a. Roll back the transaction.
   b. Map the failure to an appropriate error response using Flow 4.
5. Return result.

**Rule**: All steps 2 through 4 are coordinated by the `TransactionManager`. Services do not manage transaction commit/rollback directly.

---

## Flow 7: Monetary Amount Round-Trip

**Trigger**: Monetary amount entering or leaving the API boundary.

**Inbound (request deserialization)**:
1. Accept the `amount` field as a JSON string.
2. Parse into `DecimalAmount` preserving the input scale exactly.
3. Reject the request with validation error if the string cannot be parsed as a valid decimal.

**Outbound (response serialization)**:
1. Serialize `DecimalAmount` to JSON string with the stored scale.
2. No rounding is applied.

---

## Flow 8: Repository Ownership Filter Application

**Trigger**: Any domain repository read or write operation on user-owned resources.

**Input**: Query parameters + `Principal.user_id`.

**Rule**: Repository implementations must append `WHERE user_id = $principal_user_id` (or equivalent) to all user-scoped queries. This is not optional and must not be delegated to the service layer.

**Rationale**: Defense-in-depth ownership enforcement; prevents service-layer omissions from leaking cross-user data.

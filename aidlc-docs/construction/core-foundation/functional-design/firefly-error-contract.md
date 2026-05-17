# Firefly III Error Compatibility Contract

**Version:** 1.0  
**Firefly Target:** Latest (current: v6.5.5)  
**Date:** 2026-05-17

This document formalizes the expected error response behavior that Utopia must implement to maintain API compatibility with Firefly III clients and tooling.

---

## Error Response Structure

All error responses follow this JSON envelope:

```json
{
  "message": "<string>",
  "errors": { "<field>": ["<message>", ...], ... }
}
```

**Rules:**
- `message`: always present; human-readable summary
- `errors`: always present as an object (may be empty `{}`); populated only for HTTP 422 (validation failures)
- For auth/domain errors (401/404/500), `errors` is empty
- For validation errors (422), `message` is typically "The given data was invalid." and `errors` contains field names

---

## Error Scenarios and HTTP Status Mapping

### Authentication Layer (Protected Endpoints)

Applies to all protected routes when validation happens before reaching handler logic (middleware).

| Scenario | HTTP Status | Message Example | `errors` | Notes |
|---|---|---|---|---|
| Missing `Authorization` header | 401 | `"unauthenticated: Missing or invalid bearer token."` | `{}` | Bearer token required but not provided. |
| Malformed Bearer format (e.g., `Bearer` only, or invalid scheme) | 401 | `"token_malformed: The token format is invalid."` | `{}` | Token does not match expected format (e.g., empty after `Bearer`). Firefly returns 401 for this in most cases. |
| Token SHA-256 lookup fails (token not found in database) | 401 | `"token_not_found: The provided token was not found."` | `{}` | Token is valid format but has never been issued, or has been deleted. |
| Token status is `Revoked` | 401 | `"token_revoked: The provided token has been revoked."` | `{}` | Token exists but is marked revoked. |
| Token's user is blocked (`users.blocked = true`) | 401 | `"user_blocked: The associated user is blocked."` | `{}` | Token is valid but user account is blocked. |
| Database/cache dependency unavailable during auth | 500 | `"Internal Server Error"` or domain-specific message | `{}` | Unrecoverable dependency failure during token lookup/validation. Return 500 with generic message; log detailed error. |

**Firefly Alignment Notes:**
- Firefly v6.5.5 returns 401 for all bearer token validation failures (missing, malformed, invalid, revoked).
- No 403 Forbidden is used; authentication denial always maps to 401.
- Internal failures during auth attempt return 500.

---

### Bootstrap Token Issuance (Unprotected)

| Scenario | HTTP Status | Message Example | `errors` | Notes |
|---|---|---|---|---|
| Bootstrap key header missing | 401 | `"bootstrap_key_missing: Bootstrap key is required."` | `{}` | `X-Firefly-Bootstrap-Key` or equivalent header not provided on first-run token issuance. |
| Bootstrap key provided but does not match configured value | 401 | `"bootstrap_key_invalid: Bootstrap key is invalid."` | `{}` | Key format is valid but does not match the system's bootstrap key. |
| Bootstrap key already used (claimed by a user previously) | 401 | `"bootstrap_key_already_used: Bootstrap key has already been used."` | `{}` | Key is one-time-use; subsequent issuance attempts fail. |

**Firefly Alignment Notes:**
- Firefly v6.5.5 does not expose a bootstrap flow in the public API.
- Utopia treats bootstrap key errors as auth-adjacent (401 class) for consistency with bearer token handling.

---

### Domain Layer (Handler Level)

Applies after successful authentication when business logic is executed.

| Scenario | HTTP Status | Message Example | `errors` | Notes |
|---|---|---|---|---|
| Requested resource does not exist (e.g., token ID not found) | 404 | `"Not Found"` | `{}` | Resource lookup returns null. No reason code prefix for 404 in Firefly. |
| Resource exists but belongs to a different user (cross-user access) | 404 | `"Not Found"` | `{}` | Authorization check within handler: return 404 to avoid revealing resource existence to other users. *Same as not found* to prevent enumeration attacks. |
| Input validation fails (e.g., invalid field value, missing required field) | 422 | `"The given data was invalid."` | `{"field": ["error message"], ...}` | Validation error at handler entry. `errors` map includes all failed field validations. |
| Database transaction fails during write (UNIQUE constraint violation, foreign key, etc.) | 400 or 422 | `"The given data was invalid."` or domain-specific message | `{"field": ["Validation message"]}` | For constraint violations, treat as validation failure (422). For other write failures, return appropriate status based on semantic cause. |
| Unexpected internal error (null pointer, logic error, unhandled exception) | 500 | `"Internal Server Error"` or detailed message in debug mode | `{}` | Generic message in production; log full stack trace. |

**Firefly Alignment Notes:**
- Firefly v6.5.5 returns 404 for resource not found across all domain resources (accounts, transactions, etc.).
- Firefly returns 422 with `errors` map for input validation failures.
- For authorization denials (e.g., user tries to revoke another user's token), Firefly typically returns 404 (not 403) to avoid resource enumeration.

---

## Content Negotiation

### Request: `Accept` Header

| Accept Header | Expected Behavior |
|---|---|
| Absent | Return `application/json` |
| `application/json` | Return JSON (`application/json` content-type) |
| `application/vnd.api+json` | Return JSON formatted per JSON:API spec (if supported); else fallback to `application/json` |
| Unsupported MIME type | Return 406 Not Acceptable with error body, or fallback to JSON (depends on Firefly version) |

**Firefly Alignment Notes:**
- Firefly v6.5.5 accepts both `application/json` and `application/vnd.api+json`.
- Most responses use `application/json` content-type; `application/vnd.api+json` is legacy/optional.

### Response: `Content-Type` Header

| Response Type | Content-Type |
|---|---|
| Success (all status < 400) | `application/json; charset=utf-8` |
| Error (all status >= 400) | `application/json; charset=utf-8` |

**Firefly Alignment Notes:**
- Errors use the same `Content-Type` as success responses.
- Charset is UTF-8 for all JSON responses.

---

## Message Text Standards

### Auth Error Messages

Format: `"<reason_code>: <human-readable description>"`

Examples:
- `"unauthenticated: Missing or invalid bearer token."`
- `"token_revoked: The provided token has been revoked."`
- `"user_blocked: The associated user is blocked."`

**Firefly Alignment Notes:**
- Firefly v6.5.5 does not embed reason codes in auth error messages (only raw exception class or generic messages).
- Utopia's reason code convention is stricter and more diagnostic-friendly. Keep this convention for consistency with current design.

### Validation Error Messages

Generic form per field:
- `"The <field> field is required."`
- `"The <field> field must be a valid <type>."`
- `"The <field> has already been taken."`

Examples for a token issuance request:
- `{"label": ["The label field is required."]}`
- `{"type": ["The selected type is invalid."]}`

**Firefly Alignment Notes:**
- Firefly v6.5.5 uses Laravel-style validation messages with field placeholders and contextual messages.
- Utopia should align message phrasing to match Firefly conventions when possible.

### Domain Error Messages

Generic 404/500 messages (no reason code prefix):
- `"Not Found"` (404)
- `"Internal Server Error"` (500)

---

## Support Policy

**Compatibility Scope:**
- Utopia targets the **latest released Firefly III API** at the time of implementation and release.
- Breaking changes to Firefly error contracts will trigger a new Utopia release with explicit migration notes.
- Clients must assume that minor Firefly updates may introduce new error codes or messages; clients should handle unknown reason codes gracefully.

**Excluded from Scope:**
- Non-error business payloads (success responses, data schemas).
- Endpoint availability and method support (POST, PUT, DELETE, etc.).
- Feature-specific error codes beyond the core auth/validation/not-found/internal set.

---

## Verification Checklist

- [ ] All 401 scenarios return HTTP 401 with reason code embedded in message
- [ ] All 404 scenarios return HTTP 404 with generic "Not Found" message
- [ ] All 422 scenarios return HTTP 422 with "The given data was invalid." and `errors` map
- [ ] All 500 scenarios return HTTP 500 with generic message
- [ ] Error responses always include `errors` key (empty object if not validation)
- [ ] Content-Type is always `application/json` for errors
- [ ] No 403 status is returned (use 401 for auth denial, 404 for resource denial)
- [ ] No 400 status is returned for standard error scenarios (use 401 or 422)
- [ ] Reason codes are consistent and stable across releases
- [ ] Error messages contain no sensitive data (no token values, internal IPs, etc.)

---

## Future Enhancements

1. **Error code matrix per domain**: As accounts, transactions, budgets modules are implemented, domain-specific error codes (e.g., "account_type_invalid") may be added under 422.
2. **Localized error messages**: Message text could be localized per Accept-Language; currently English only.
3. **Rate limiting**: If rate limiting is added, use 429 Too Many Requests with appropriate error body.

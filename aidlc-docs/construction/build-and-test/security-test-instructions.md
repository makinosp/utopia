# Security Test Instructions

## Purpose
Validate security controls required by the enabled Security Baseline extension and UOW-01 design.

## Scope
- Authentication and authorization guard behavior
- Secret handling and log redaction
- Dependency vulnerability scanning
- Container image and configuration checks
- Input validation and error contract behavior

## Prerequisites
- Application starts successfully with .env configuration
- Postgres service is running
- Test token is available for protected endpoint checks

## Test Steps

### 1. Dependency Vulnerability Scan
```bash
cargo install cargo-audit
cargo audit
```

Expected:
- No unpatched critical vulnerabilities

### 2. Secret Leakage and Redaction Check
```bash
# Run app and generate auth failures, then inspect logs
cargo run
```

Validate in logs:
- No raw token values
- No Authorization header content
- No bootstrap secret values
- Only allowed fields such as request_id, reason_code, and user_id

### 3. Authentication Guard Checks
Use API calls to verify:
- Missing Authorization header returns 401
- Malformed Bearer token returns 401 with reason code token_malformed
- Revoked token returns 401 with reason code token_revoked

### 4. Bootstrap One-Time Key Validation
Verify:
- First valid bootstrap call succeeds
- Subsequent bootstrap calls with same key are rejected
- Raw bootstrap key never appears in logs or responses

### 5. HTTP Security Headers Check
```bash
curl -I https://<host>
```

Required headers:
- Content-Security-Policy
- Strict-Transport-Security
- X-Content-Type-Options
- X-Frame-Options
- Referrer-Policy

### 6. Container and Image Configuration Checks
Validate compose and Dockerfile:
- No latest tags
- No secrets hardcoded in compose file
- Runtime user is non-root in final image

### 7. k6 Auth Security Validation (UOW-05)
```bash
k6 run -e BASE_URL=http://localhost:3000 k6/auth.ts
```

Validates:
- Unauthenticated requests are rejected with 401
- Invalid bootstrap key is rejected
- Revoked tokens are rejected with `token_revoked` reason code
- Token values are not leaked in error responses

### 8. Input Validation Checks
```bash
# Test malformed JSON handling
curl -s -X POST http://localhost:3000/api/v1/accounts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN_A}" \
  -d 'invalid-json'

# Test oversized payload handling
curl -s -X POST http://localhost:3000/api/v1/transactions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN_A}" \
  -d '{"type":"'$(python3 -c "print('A'*10000)")'"}'
```

Expected:
- Malformed JSON returns 400 with error envelope
- Oversized payload returns 413 or 400

## Pass Criteria
- All required checks pass
- No critical or high-risk security findings remain unresolved
- Security baseline controls are verifiably active
- k6 auth security scenarios all pass

## Remediation Flow
If any check fails:
1. Record failing control and evidence
2. Apply fix in source or configuration
3. Re-run only failed test
4. Re-run full security test checklist

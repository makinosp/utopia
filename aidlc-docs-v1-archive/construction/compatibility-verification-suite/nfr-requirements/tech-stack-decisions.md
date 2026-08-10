# Tech Stack Decisions — UOW-05: Compatibility Verification Suite

## Overview
This document records the technology choices made for the Compatibility Verification Suite, with rationale for each decision.

## Runtime and Language

### Seed Data Generation
- **Language**: TypeScript
- **Runtime**: Bun
- **Rationale**: Bun provides native TypeScript execution without a compilation step, fast startup, and full npm compatibility. TypeScript enables type definitions to be shared with k6 test scripts.

### k6 Test Scripts
- **Language**: TypeScript
- **Runtime**: k6 (Go-based, executes TypeScript via bundled Babel)
- **Rationale**: k6 natively supports TypeScript test scripts. Using the same language as seed generation enables type sharing and reduces context switching.

## Database

### Test Database
- **Engine**: PostgreSQL (via Docker Compose, same as development)
- **Rationale**: Utopia uses PostgreSQL as its primary datastore. Tests must validate against the same engine to ensure compatibility.

### Database Client for Seed Scripts
- **Library**: `pg` (node-postgres)
- **Rationale**: Most mature PostgreSQL client for TypeScript/Bun; supports connection pooling and parameterized queries.

## Containerization

### Docker Compose
- **k6 service**: Official `grafana/k6` image
- **Rationale**: Official image maintained by Grafana Labs; minimal configuration required.

### Service Dependencies
- k6 service depends on `app` service health check passing.
- Rationale: Ensures the application stack is fully initialized before tests execute.

## CI Integration

### GitHub Actions
- **Trigger**: On every pull request
- **Steps**: Start stack → Wait for health → Seed data → Run k6 → Report results
- **Rationale**: Validates compatibility on all changes; prevents regressions.

### CI Artifacts
- JSON summary report: stored as CI artifact for automated gate evaluation.
- HTML report: stored as CI artifact for manual investigation on failure.

## Output Format

### k6 Output
- **JSON**: Machine-parseable summary for CI gate (pass/fail counts, timing metrics).
- **HTML**: Human-readable report with request/response details for debugging.
- **Rationale**: Dual output supports both automated CI decisions and manual developer investigation.

## Dependency Management

### Seed Scripts
- **Package manager**: Bun (`bun install`)
- **Configuration**: `package.json` at project root or `scripts/seed/package.json`
- **Rationale**: Bun's package manager is fast and fully npm-compatible.

### k6 Scripts
- **No external dependencies**: k6 scripts use only built-in modules (http, check, sleep).
- **Rationale**: Minimizes dependency management overhead for test scripts.

## File Structure

```
scripts/
├── seed/
│   ├── index.ts              # Entry point: truncate + seed
│   ├── accounts.ts           # Account fixture definitions
│   ├── transactions.ts       # Transaction fixture definitions
│   └── types.ts              # Shared type definitions
├── k6/
│   ├── auth.ts               # Authentication endpoint tests
│   ├── accounts.ts           # Account endpoint tests
│   ├── transactions.ts       # Transaction endpoint tests
│   └── fixtures/
│       ├── auth.json         # Expected auth responses
│       ├── accounts.json     # Expected account responses
│       └── transactions.json # Expected transaction responses
└── package.json              # Seed script dependencies
```

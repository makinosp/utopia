# Logical Components — UOW-05: Compatibility Verification Suite

## Overview
This document describes the logical components and infrastructure elements that comprise the Compatibility Verification Suite.

## Test Execution Components

### k6 Runner
- **Role**: Executes TypeScript test scripts against the Utopia API.
- **Runtime**: k6 (Go-based) with built-in TypeScript support via Babel.
- **Configuration**: 1 VU, shared iteration mode, no throttling.
- **Image**: `grafana/k6:latest` in Docker Compose.

### Test Harness (`k6/harness.ts`)
- **Role**: Shared utility module for all test scripts.
- **Responsibilities**:
  - Base URL configuration (from `APP_BASE_URL` environment variable)
  - Authentication token retrieval (login once, reuse across tests)
  - Common response validation helpers
  - Dynamic field exclusion for strict mode comparison

### Domain Test Scripts
- `k6/auth.ts`: Authentication endpoint tests (token issuance, revocation, unauthenticated rejection)
- `k6/accounts.ts`: Account endpoint tests (list, get, create, update, delete)
- `k6/transactions.ts`: Transaction endpoint tests (list, get, create, update, delete, list by account)

## Fixture Components

### Fixture Files (`k6/fixtures/`)
- `k6/fixtures/auth.json`: Expected authentication response structures
- `k6/fixtures/accounts.json`: Expected account response structures
- `k6/fixtures/transactions.json`: Expected transaction response structures
- **Format**: JSON schema with wildcard support for dynamic fields
- **Usage**: Loaded by test scripts for strict mode comparison

### Dynamic Field Handling
- **Approach**: Schema-based wildcard matching
- **Dynamic fields excluded**: `created_at`, `updated_at`, `id`, `request_id`
- **Implementation**: k6's `check()` function with JSON schema validation
- **Rationale**: Validates response structure and types while ignoring values that change between runs. More robust than stripping (which would miss unexpected fields) or JSONPath (which requires brittle path lists).

## Seed Data Components

### Seed Generator (`scripts/seed/index.ts`)
- **Role**: Generates deterministic test data and loads it into the database.
- **Runtime**: Bun (TypeScript native execution).
- **Responsibilities**:
  - Truncate all tables (`TRUNCATE ... CASCADE`)
  - Insert seed data for accounts, transactions, and test users
  - Output summary of seeded records
- **Execution**: `bun run scripts/seed/index.ts`

### Seed Type Definitions (`scripts/seed/types.ts`)
- **Role**: Shared TypeScript interfaces for seed data structures.
- **Types**:
  - `AccountSeed`: name, accountType, currencyCode, balance
  - `TransactionSeed`: accountId, amount, description, date, category
  - `UserSeed`: email, password (synthetic, non-production)
- **Usage**: Imported by both seed generator and k6 test scripts for type consistency.

### Seed Domain Modules
- `scripts/seed/accounts.ts`: Account fixture definitions and generation logic
- `scripts/seed/transactions.ts`: Transaction fixture definitions and generation logic

## Infrastructure Components

### Docker Compose Integration
- **Service name**: `k6`
- **Depends on**: `app` service (health check passing)
- **Volumes**: `./k6:/scripts/k6` (test scripts), `./scripts/seed:/scripts/seed` (seed scripts)
- **Environment**: `APP_BASE_URL=http://app:8080`, `DATABASE_URL` (for seed script)

### CI Pipeline (GitHub Actions)
- **Trigger**: On every pull request
- **Steps**:
  1. Start application stack (`docker compose up -d`)
  2. Wait for health check (`GET /health` → 200 OK)
  3. Run seed data (`bun run scripts/seed/index.ts`)
  4. Run k6 suite (`docker compose run k6 run /scripts/k6/harness.ts`)
  5. Upload artifacts (JSON + HTML reports)
  6. Tear down stack (`docker compose down`)

### Output Artifacts
- **JSON report**: `k6-results.json` — pass/fail counts, timing metrics, custom tags
- **HTML report**: `k6-results.html` — human-readable report with request/response details
- **CI gate**: JSON report parsed for pass/fail determination; PR blocked on failure

## Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    GitHub Actions CI                     │
│                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────────────┐   │
│  │  Start   │───▶│  Health  │───▶│  Seed Data       │   │
│  │  Stack   │    │  Check   │    │  (Bun/TS)        │   │
│  └──────────┘    └──────────┘    └────────┬─────────┘   │
│                                           │             │
│                                           ▼             │
│                                    ┌──────────────┐     │
│                                    │  PostgreSQL  │     │
│                                    │  (test DB)   │     │
│                                    └──────┬───────┘     │
│                                           │             │
│  ┌────────────────────────────────────────┘             │
│  │                                                      │
│  ▼                                                      │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐           │
│  │  k6      │───▶│  Utopia  │───▶│  Assert  │           │
│  │  Runner  │    │  API     │    │  Results │           │
│  └──────────┘    └──────────┘    └────┬─────┘           │
│                                       │                 │
│                                       ▼                 │
│                              ┌──────────────┐           │
│                              │  JSON + HTML │           │
│                              │  Reports     │           │
│                              └──────────────┘           │
└─────────────────────────────────────────────────────────┘
```

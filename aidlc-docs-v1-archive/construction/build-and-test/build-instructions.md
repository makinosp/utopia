# Build Instructions

## Prerequisites
- Build tool: Rust toolchain 1.86.x and Cargo
- Container tools: Docker Engine 25+ and Docker Compose v2
- Optional observability stack: Prometheus and Grafana containers from compose profile
- k6 testing: k6 CLI (v0.50+) or Docker k6 image (grafana/k6:latest)
- Seed runtime: Bun runtime (v1.0+) for seed data generator
- TypeScript tooling: pnpm (v9+) for oxlint and oxfmt
- System requirements: Linux host, 4+ CPU cores, 8+ GB RAM, 20+ GB free disk

## Required Environment Variables
Create a runtime environment file before build and run:
- APP_PORT
- LOG_LEVEL
- DATABASE_URL
- ARGON2_MEMORY_COST
- ARGON2_TIME_COST
- ARGON2_PARALLELISM
- TOKEN_CACHE_TTL_SECS
- NEGATIVE_TOKEN_CACHE_TTL_SECS
- TOKEN_CACHE_MAX_CAPACITY
- BOOTSTRAP_KEY
- BOOTSTRAP_USER_EMAIL
- APP_BASE_URL (for k6 tests, e.g. http://localhost:3000)
- K6_OUTPUT_DIR (optional, defaults to k6-results/)

Reference template: .env.example

## Build Steps

### 1. Install Dependencies
```bash
rustup toolchain install 1.86.0
rustup default 1.86.0
cargo --version
docker --version
docker compose version
```

### 2. Configure Environment
```bash
cp .env.example .env
# Edit .env values for your environment
```

### 3. TypeScript Lint and Format
All TypeScript source files (scripts/, k6/, and config files) must pass formatting
and lint checks before merge.

```bash
# Install dependencies (first time only)
pnpm install

# Check formatting
pnpm fmt:check

# Lint
pnpm lint

# Fix formatting issues
pnpm fmt

# Fix auto-fixable lint issues
pnpm lint:fix
```

### 4. Build Application
```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo build --release
```

### 5. Build Container Image
```bash
docker build -t utopia-api:0.1.0 .
```

### 6. Verify Build Output
Expected output:
- Cargo build completes with Finished profile messages
- Docker build completes with tagged image utopia-api:0.1.0

Build artifacts:
- target/debug/utopia
- target/release/utopia
- Docker image utopia-api:0.1.0

### 7. Build Seed Data Generator (UOW-05)
```bash
cd scripts/seed
bun install
cd ../..
```

### 8. Verify k6 Installation (UOW-05)
```bash
# Option A: Native k6
k6 version

# Option B: Docker k6 image
docker pull grafana/k6:latest
```

## CI Execution Mapping (Phase 1)

Implemented workflow:
- `.github/workflows/ci-phase1.yml`

Trigger policy:
- pull_request: all branches
- push: main
- workflow_dispatch: manual

Blocking checks in Phase 1:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --locked`
- `cargo build --release --locked`
- `cargo test --locked`

Advisory-only checks in Phase 1:
- `cargo audit` (non-blocking, warning-only)

Branch protection expected required check:
- `quality-build-test`

## CI Execution Mapping (Phase 2 — k6 Compatibility Verification Suite)

Implemented workflow:
- `.github/workflows/ci-phase2.yml` (reclassified from the former
  `compatibility-check.yml` per user request on 2026-07-07)

Trigger policy:
- pull_request: main
- push: main

Blocking checks in Phase 2:
- k6 auth tests (6 scenarios)
- k6 accounts tests (7 scenarios)
- k6 transactions tests (8 scenarios)
- Firefly-III envelope validation (list/single/error envelope, 204 No Content,
  401 Unauthorized, pagination consistency)
- k6 result artifact upload (retention 30 days)

Prerequisites for Phase 2:
- Docker Compose stack (utopia-api, postgres, caddy) started in CI
- Seed data loaded via `scripts/seed/index.ts` before each test group
- `BOOTSTRAP_KEY` provided via GitHub secret (fallback CI test key)

## Troubleshooting

### Dependency Resolution Failures
Cause:
- Missing network access or corrupted local cache

Solution:
```bash
cargo update
cargo clean
cargo build
```

### SQLx Migration or DB Connectivity Failures
Cause:
- DATABASE_URL invalid or Postgres unavailable

Solution:
```bash
docker compose -f docker/docker-compose.yml up -d postgres
cargo build
```

### TLS Configuration Failures
Cause:
- DATABASE_URL missing sslmode=require

Solution:
- Ensure DATABASE_URL includes sslmode=require
- Re-run cargo build after updating .env

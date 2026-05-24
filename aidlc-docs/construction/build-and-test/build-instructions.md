# Build Instructions

## Prerequisites
- Build tool: Rust toolchain 1.86.x and Cargo
- Container tools: Docker Engine 25+ and Docker Compose v2
- Optional observability stack: Prometheus and Grafana containers from compose profile
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

### 3. Build Application
```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo build --release
```

### 4. Build Container Image
```bash
docker build -t utopia-api:0.1.0 .
```

### 5. Verify Build Output
Expected output:
- Cargo build completes with Finished profile messages
- Docker build completes with tagged image utopia-api:0.1.0

Build artifacts:
- target/debug/utopia
- target/release/utopia
- Docker image utopia-api:0.1.0

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

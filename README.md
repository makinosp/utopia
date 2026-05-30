# Utopia

Utopia is a lightweight, self-hostable personal finance API with partial compatibility with Firefly-III.

## Overview

Utopia implements a subset of the Firefly-III API surface to provide accounts, token management, transaction journals, and Prometheus metrics. This repository contains the server implementation, database migrations, and supporting infrastructure for local development and observability.

## Supported API (high level)

- Accounts API
- Token issuance and management (personal access tokens)
- Bootstrap token issuance (initial provisioning)
- Transactions / journals
- Prometheus `/metrics` endpoint

See the full API contract in [openapi.yaml](openapi.yaml).

## Quick start

Prerequisites: Docker Engine and Docker Compose (or a Rust toolchain for local builds). For exact toolchain and build instructions see [aidlc-docs/construction/build-and-test/build-instructions.md](aidlc-docs/construction/build-and-test/build-instructions.md).

Start with Docker Compose:

```bash
cp .env.example .env
docker compose -f docker/docker-compose.yml up --build
```

Run locally (development):

```bash
cp .env.example .env
# adjust DATABASE_URL as needed
cargo build
cargo run
```

Verify the service (example):

```bash
curl -fsS http://localhost:3000/metrics | head -n 20
```

## Configuration

Runtime configuration is provided via environment variables. See [.env.example](.env.example) for required keys and sensible defaults used for local development.

## Build & test

Refer to [aidlc-docs/construction/build-and-test/build-instructions.md](aidlc-docs/construction/build-and-test/build-instructions.md) for recommended toolchain versions, CI checks, and detailed build and test commands.

## Repository layout (high level)

- src/ — application source code (server, handlers, core logic)
- migrations/ — database migrations
- docker/ — docker-compose and container helpers
- aidlc-docs/ — architectural and planning documents
- tests/ — integration and unit tests

For a detailed code layout and design rationale see aidlc-docs/.

## Documentation

Design artifacts, plans, and implementation notes live in aidlc-docs/. Use those documents for architecture, NFRs, and construction plans.

## License

This project is licensed under the BSD-3-Clause license. See [LICENSE](LICENSE) for details.

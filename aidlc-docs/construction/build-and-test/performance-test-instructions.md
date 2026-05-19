# Performance Test Instructions

## Purpose
Validate UOW-01 auth and token endpoints against the approved NFR baseline.

## Performance Targets
- Auth validation latency: p95 <= 100 ms
- Sustained throughput: 100 authenticated requests/second
- Burst throughput: 150 requests/second up to 60 seconds
- Error thresholds:
  - Auth failure rate alert at > 5 percent for 10 minutes
  - HTTP 5xx alert at > 1 percent for 10 minutes

## Test Environment Setup

### 1. Start Runtime
```bash
docker compose -f docker/docker-compose.yml up -d postgres
docker compose -f docker/docker-compose.yml --profile observability up -d prometheus grafana loki promtail node-exporter postgres-exporter
cargo run
```

### 2. Prepare Test Data
- Create bootstrap token once
- Create at least one active token for load execution

### 3. Configure Load Test Parameters
- Duration: 15 minutes
- Ramp-up: 60 seconds
- Virtual users: 100 baseline, 150 burst phase

## k6 Example Scripts

### Auth Validation Load
```bash
k6 run -e BASE_URL=http://localhost:3000 -e TOKEN=<active_token> ./perf/auth-load.js
```

### Stress Ramp
```bash
k6 run -e BASE_URL=http://localhost:3000 -e TOKEN=<active_token> ./perf/auth-stress.js
```

## Result Evaluation
Collect:
- k6 output summary (latency and rate)
- /metrics values for auth_validation_latency_ms and http_5xx_total
- Grafana dashboard snapshots for test window

Pass criteria:
- p95 auth validation latency <= 100 ms
- sustained >= 100 rps with acceptable error budget
- burst phase does not trigger prolonged 5xx instability

## If Performance Fails
1. Inspect cache hit/miss and dependency failure counters
2. Inspect Postgres query latency and connection pool saturation
3. Tune cache TTL and pool settings
4. Re-run same scenario and compare results

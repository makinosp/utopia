# Infrastructure Design - Core Foundation (UOW-01)

## Scope
This document maps the Core Foundation logical components to a concrete self-hosted infrastructure profile for the initial deployment.

## Approved Deployment Decisions
- Primary environment: home server with container runtime.
- Packaging model: Docker Compose stack.
- Database model: PostgreSQL in a separate container on the same host.
- Durability baseline: daily full backups plus hourly WAL archive/incremental coverage.
- Secret injection: environment variables provided by the Compose runtime.
- Ingress: reverse proxy with TLS termination and internal app port.
- Observability: logs, Prometheus metrics, and Grafana dashboards.
- Alert delivery: email.
- Shared infrastructure scope: none beyond this unit in the current phase.
- Environment separation: single personal-use environment.

## Runtime Topology

| Layer | Service | Role | Exposure |
|---|---|---|---|
| Edge | Caddy | TLS termination, reverse proxy, security headers, access logging | Public ports 80/443 only |
| Application | utopia-api | Core Foundation HTTP API, auth middleware, business logic, metrics endpoint | Internal port only |
| Data | PostgreSQL | Source-of-truth persistence for users, tokens, and transactional state | Internal port only |
| Backup | pgBackRest job container | Full backups, WAL archive management, restore tooling | Internal only |
| Observability | Prometheus | Metrics collection and alert rule evaluation | Internal only |
| Observability | Grafana | Dashboards and email alert delivery | Internal only |
| Observability | Loki | Centralized log store | Internal only |
| Observability | Promtail | Log shipping from containers and reverse proxy access logs | Internal only |
| Host telemetry | node_exporter | Host CPU, memory, disk, and filesystem metrics | Internal only |
| Database telemetry | postgres_exporter | PostgreSQL health and performance metrics | Internal only |

## Logical-to-Physical Mapping

| Logical Component | Physical Placement | Supporting Infrastructure |
|---|---|---|
| Auth Middleware Facade | utopia-api container | Caddy ingress, Prometheus scrape target |
| Auth Validator | utopia-api container | PostgreSQL, in-process cache |
| Token Cache | utopia-api container memory | No external cache in this phase |
| Error Mapper | utopia-api container | Caddy request forwarding and standardized 5xx handling |
| Metrics Emitter | utopia-api container | Prometheus, Grafana |
| Audit Logger | utopia-api container | JSON stdout log stream, Promtail, Loki |
| Config Validator | utopia-api container startup path | Compose env injection, startup health checks |
| Repository Read/Write Interfaces | utopia-api container | PostgreSQL over internal TLS |

## Compute Infrastructure

### Caddy Reverse Proxy
- Terminates TLS for all public traffic.
- Forwards requests to the internal utopia-api service over the Docker internal network.
- Adds required HTTP security headers for HTML-serving endpoints and preserves HSTS for HTTPS traffic.
- Writes structured access logs for all external traffic so the network intermediary remains auditable.

### Utopia API Service
- Runs as a single application container because all Core Foundation logical components remain in-process.
- Exposes one internal HTTP port plus one Prometheus metrics endpoint.
- Uses a bounded process-local positive token cache with explicit invalidation on token revoke or security configuration rotation.
- Fails closed when PostgreSQL is unavailable and emits structured failure telemetry rather than retrying in the request path.

### Background and Async Strategy
- No external message broker is introduced in this phase.
- Fire-and-forget tasks stay in-process and must never become correctness-critical.
- If a future unit requires durable async work, it should introduce a separate queue as a deliberate follow-up design decision.

## Storage Infrastructure and Persistence Topology

### PostgreSQL
- Deploy PostgreSQL as a dedicated Compose service on the same host, isolated from the public network.
- Use a host-mounted persistent volume located on an encrypted filesystem.
- Enable TLS for all application-to-database connections, even on the internal Docker network, to satisfy in-transit encryption requirements.
- Restrict database access to the utopia-api service and backup tooling only.
- Keep database credentials out of the repository and inject them at runtime from host-managed environment configuration.

### Encryption Baseline
- Use host-level disk encryption such as LUKS/dm-crypt for PostgreSQL data, backup repositories, and observability data volumes.
- Use TLS 1.2+ at the public edge and for PostgreSQL client/server traffic.
- Store certificates and private keys outside version control with root-owned permissions.

### Backup and Recovery
- Use pgBackRest in the same Compose stack for backup orchestration.
- Run one daily full backup of PostgreSQL data.
- Archive WAL continuously and verify archive integrity at least hourly so recovery points remain granular.
- Keep the backup repository on an encrypted host path separate from the active PostgreSQL volume.
- Replicate the backup repository to secondary offline or removable storage on a scheduled basis as an operational runbook task.
- Maintain a manual restore runbook aligned with the approved NFR requirement for documented recovery steps.

## Networking and Ingress Architecture

### Network Segmentation
- Use two Docker networks:
  - `edge`: Caddy only, attached to host-published 80/443.
  - `internal`: utopia-api, PostgreSQL, backup tooling, and observability services.
- Do not publish PostgreSQL, Prometheus, Loki, or Grafana directly to the public internet.
- Permit host firewall ingress only on TCP 80 and 443.

### Traffic Flow
1. External client connects to Caddy over HTTPS.
2. Caddy applies TLS, access logging, and security header policy.
3. Caddy forwards traffic to utopia-api on the internal Docker network.
4. Utopia API accesses PostgreSQL over internal TLS.
5. Logs flow from Caddy and utopia-api to Promtail and then to Loki.
6. Metrics flow from utopia-api, node_exporter, and postgres_exporter to Prometheus.
7. Grafana reads Prometheus and Loki for dashboards and sends alert emails.

## Monitoring, Logging, and Alerting Stack

### Logging
- Utopia API emits JSON structured logs to stdout with timestamp, request_id, log level, endpoint, status_code, auth_outcome, and user_id when available.
- Caddy access logs are enabled and shipped to Loki to satisfy network intermediary logging requirements.
- Promtail collects container logs and forwards them to Loki.
- Loki retains logs for at least 90 days on encrypted storage.
- The application container has no filesystem-level access to Loki data, preventing self-deletion of centralized logs.

### Metrics
- Prometheus scrapes:
  - utopia-api metrics endpoint
  - node_exporter
  - postgres_exporter
- Required auth and service metrics from prior NFR artifacts are treated as mandatory instrumentation targets.

### Alerting
- Grafana evaluates alert rules backed by Prometheus and delivers email notifications.
- Initial alert set includes:
  - auth failure rate above 5 percent for 10 minutes
  - HTTP 5xx rate above 1 percent for 10 minutes
  - p95 auth validation latency above 100ms for 15 minutes
  - PostgreSQL unavailable or replication/archive check failure
  - repeated authentication dependency failures
- Alert routing starts with email only, matching the approved plan.

## Secrets and Configuration Handling
- Inject runtime secrets as environment variables from host-managed Compose configuration.
- Keep the environment file outside the repository with owner-only read permissions.
- Rotate application secrets and signing values at least quarterly.
- Restart affected services through a controlled maintenance action after secret rotation.
- Never log secret values, raw tokens, authorization headers, or direct personal identifiers.

## Shared Infrastructure Boundaries and Isolation
- This unit remains self-contained and does not establish a shared platform baseline for future units.
- Shared use is intentionally limited to the local reverse proxy and observability stack inside this single deployment.
- Future units may either join the same home-server host or deploy separately, but no coupling is assumed now.
- Data, metrics, and logs remain identifiable by service labels so later units can be onboarded without retrofitting all current pipelines.

## Operational Constraints and Deferred Decisions
- High availability, multi-node orchestration, and replica routing are explicitly deferred.
- No external cache, message queue, or service mesh is introduced in this phase.
- This design optimizes for secure simplicity and an upgrade path rather than immediate horizontal scale.

## Security Baseline Compliance Summary

| Rule | Status | Infrastructure Design Decision |
|---|---|---|
| SECURITY-01 | Compliant | Encrypted host volumes plus TLS 1.2+ for ingress and PostgreSQL traffic |
| SECURITY-02 | Compliant | Caddy access logging enabled and shipped to centralized log storage |
| SECURITY-03 | Compliant | Structured application logs centralized through Promtail and Loki |
| SECURITY-04 | Compliant | Reverse proxy sets required HTTP security headers for HTML-serving endpoints |
| SECURITY-05 | N/A | API parameter validation is enforced in application code generation, not by infrastructure topology |
| SECURITY-06 | N/A | No cloud IAM policy surface exists in the selected self-hosted home-server profile |
| SECURITY-07 | Compliant | Only 80/443 are public; all other services remain internal-only |
| SECURITY-08 | N/A | Application-layer authorization is enforced in code and service design rather than infrastructure layout |
| SECURITY-09 | Compliant | No default credentials, minimal service exposure, and production-safe error exposure posture |
| SECURITY-10 | N/A | Dependency pinning and SBOM generation are handled in code generation and build/test stages |
| SECURITY-11 | Compliant | Defense-in-depth preserved through TLS, validation seams, access control boundaries, and alerting |
| SECURITY-12 | Compliant | Secrets stay outside source control and runtime credential handling is defined |
| SECURITY-13 | N/A | Artifact integrity and auditability are build/deployment concerns outside this stage's infrastructure mapping scope |
| SECURITY-14 | Compliant | Email alerts, 90-day log retention, and centralized dashboards are defined |

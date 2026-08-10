# Deployment Architecture - Core Foundation (UOW-01)

## Topology Summary
The initial deployment is a single-host home-server stack orchestrated with Docker Compose. Public traffic is terminated at Caddy, application logic runs in a single utopia-api container, PostgreSQL runs in a separate private container, and the observability stack remains private to the host.

## Deployment Units

| Deployment Unit | Container Count | Persistence | Network Zone |
|---|---|---|---|
| Edge proxy | 1 | Access logs, TLS certificates | edge + internal |
| Application | 1 | Stateless except in-process cache | internal |
| Database | 1 | PostgreSQL data volume | internal |
| Backup tooling | 1 scheduled job/service | Backup repository volume | internal |
| Observability | 5 | Prometheus TSDB, Grafana state, Loki log store | internal |

## Host Layout

### Public Surface
- Caddy publishes TCP 80 and 443 on the host.
- The host firewall denies inbound access to all other ports.

### Private Surface
- utopia-api listens only on the internal Compose network.
- PostgreSQL listens only on the internal Compose network with TLS enabled.
- Grafana, Prometheus, Loki, and exporters are reachable only from the internal network or local host administration path.

## Persistent Storage Layout

| Data Class | Storage Location | Protection Strategy |
|---|---|---|
| PostgreSQL primary data | Encrypted host-mounted volume | LUKS/dm-crypt, internal-only access |
| WAL archive and backups | Separate encrypted host-mounted backup volume | pgBackRest, daily full backup, hourly archive verification |
| Reverse proxy certificates | Root-owned host path | Restricted permissions, excluded from repository |
| Prometheus metrics data | Encrypted host-mounted volume | Internal-only access |
| Loki logs | Encrypted host-mounted volume | 90-day retention, application cannot mutate store |
| Grafana state | Encrypted host-mounted volume | Internal-only access |

## Request Path Architecture
1. A client request reaches Caddy over HTTPS.
2. Caddy records an access log entry and enforces edge security headers.
3. Caddy proxies the request to utopia-api over the internal network.
4. Utopia API performs auth validation, cache lookup, repository access, and response mapping.
5. When persistence is required, utopia-api connects to PostgreSQL over TLS.
6. Application logs and reverse proxy logs are shipped to Loki through Promtail.
7. Metrics are scraped by Prometheus and surfaced in Grafana dashboards.

## Failure Behavior

### Database Failure
- Protected operations fail closed.
- Utopia API emits structured error logs and failure metrics.
- Grafana email alerts notify maintainers when database unavailability persists.

### Reverse Proxy Failure
- Public traffic becomes unavailable.
- Internal services remain intact for diagnosis.
- Recovery is performed by restarting or reloading the Caddy service on the same host.

### Observability Failure
- Application traffic continues, but telemetry visibility degrades.
- Logs remain available locally in container stdout until shippers recover.
- Alerting on observability pipeline health should detect prolonged ingestion failure.

## Administration Model
- Deployments are executed from the home server by a trusted operator only.
- Runtime environment files are stored outside version control.
- Backup restore, secret rotation, and certificate renewal are documented operational tasks.
- No multi-tenant access model is assumed in this phase.

## Future Expansion Path
- Add a dedicated replica PostgreSQL node when scale triggers are met.
- Split observability onto a separate host if resource contention appears.
- Introduce a shared ingress or platform layer only after a later unit explicitly approves that coupling.

## Infrastructure Acceptance Criteria
- The full stack can be started by Docker Compose on one home-server host.
- Only HTTPS traffic is public.
- PostgreSQL data and log stores reside on encrypted persistent storage.
- Backup automation supports full restore plus point-in-time recovery from WAL archives.
- Metrics, dashboards, centralized logs, and email alerts are present before production-like use.

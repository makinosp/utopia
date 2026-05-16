# Infrastructure Design Plan - Core Foundation (UOW-01)

## Objective
Map the Core Foundation logical components to practical self-hosted infrastructure choices for secure, reliable operation in the initial deployment profile.

## Context Inputs
- Unit: UOW-01 Core Foundation
- Functional design:
  - aidlc-docs/construction/core-foundation/functional-design/business-logic-model.md
  - aidlc-docs/construction/core-foundation/functional-design/business-rules.md
- NFR design:
  - aidlc-docs/construction/core-foundation/nfr-design/nfr-design-patterns.md
  - aidlc-docs/construction/core-foundation/nfr-design/logical-components.md
- NFR requirements:
  - aidlc-docs/construction/core-foundation/nfr-requirements/nfr-requirements.md

## Infrastructure Design Checklist
- [ ] Define deployment environment and runtime topology
- [ ] Define compute infrastructure mapping for auth and API middleware path
- [ ] Define storage infrastructure and persistence topology
- [ ] Define messaging/async processing strategy (if needed)
- [ ] Define networking and ingress architecture
- [ ] Define monitoring and logging infrastructure stack
- [ ] Define shared infrastructure boundaries and isolation strategy
- [ ] Generate infrastructure-design.md
- [ ] Generate deployment-architecture.md
- [ ] Request approval to proceed to Code Generation

## Planning Questions

Please fill all [Answer]: fields.

## Question 1
What should be the primary deployment environment for initial scope?

A) Local bare-metal Linux host
B) Single VPS (self-managed)
C) Home server with container runtime
D) Managed cloud VM (IaaS) with self-managed app stack
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 2
How should compute be packaged for deployment?

A) Single native binary process with systemd
B) Single Docker container
C) Docker Compose stack with app + dependencies
D) Kubernetes deployment from day one
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 3
Which database deployment model should be used initially?

A) PostgreSQL on same host as app
B) PostgreSQL in separate container on same host
C) Managed PostgreSQL service
D) PostgreSQL on dedicated separate VM
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 4
What persistence durability baseline should be required?

A) Daily backups only
B) Daily full + hourly WAL/incremental backups
C) Continuous replication to standby + daily backups
D) Snapshot-based only (no WAL strategy)
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 5
How should secrets be provided to runtime in initial deployment?

A) Environment variables from systemd/compose
B) Encrypted env file + runtime decryption step
C) External secret manager integration now
D) File-mounted secrets only
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 6
What ingress/networking architecture should be used?

A) Direct app port exposure with host firewall rules
B) Reverse proxy (Nginx/Caddy) + TLS termination + app internal port
C) API gateway service + internal app network
D) Service mesh-based ingress
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 7
How should observability infrastructure be deployed initially?

A) Application logs only (JSON logs to file)
B) Logs + Prometheus metrics endpoint (no dashboard)
C) Logs + Prometheus + Grafana dashboards
D) Full stack (logs, metrics, tracing, alertmanager)
X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 8
What alert delivery channel should be targeted in initial setup?

A) None (manual checks only)
B) Email alerts
C) Chat webhook alerts (Slack/Discord)
D) Email + chat webhook dual channel
X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 9
Is shared infrastructure needed now across future units?

A) No shared layer now; each unit remains process-local
B) Shared PostgreSQL and reverse proxy only
C) Shared observability + security layer from this unit onward
D) Full shared platform baseline for all future units now
X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question 10
What environment separation is required at this phase?

A) Single environment only (dev/prod combined for personal use)
B) Separate local dev and one production environment
C) Dev + staging + production
D) Ephemeral preview environments per branch
X) Other (please describe after [Answer]: tag below)

[Answer]: A

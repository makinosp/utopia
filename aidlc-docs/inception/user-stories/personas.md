# Personas

## Overview

Three personas have been identified for this project based on the requirements and planning answers.

---

## Persona A — Household End User

**Name**: Maya  
**Role**: Household budget manager  
**Segment**: Consumer / indirect API user (via client apps)

### Description
Maya uses an existing Firefly-III compatible client (such as Waterfly-III) on her smartphone to track household income and expenses. She does not interact directly with the API — the client app is her only interface. She expects the system to faithfully record transactions, show accurate account balances, and reflect budget limits without requiring manual reconciliation.

### Goals
- Record daily income and expense transactions quickly through her client app.
- View current account balances and budget status at any time.
- Trust that her financial data is accurate, complete, and safe.

### Pain Points
- Client app breaks or behaves unexpectedly when API responses are incompatible.
- Incorrect balance calculations lead to distrust of the system.
- Data loss or inconsistency after server restarts or upgrades.

### Technical Context
- Does not configure the API directly.
- Uses an existing Firefly-III compatible client; expects transparent compatibility.
- Personal/household scale — not enterprise volume.

---

## Persona B — Self-Hosting Admin / Operator

**Name**: Kenji  
**Role**: Self-hosting operator and system administrator  
**Segment**: Technical user / deployment owner

### Description
Kenji hosts the API service on his own server (VPS or home lab). He is responsible for installation, configuration, upgrades, and backup. He needs clear API documentation, predictable deployment behavior, and confidence that the system is secure by default without requiring extensive hardening after installation.

### Goals
- Deploy and configure the API service with minimal friction.
- Keep the service running reliably across version upgrades.
- Maintain security and data integrity without becoming a security expert.

### Pain Points
- Opaque configuration that requires trial-and-error to get right.
- Breaking API changes between versions that disrupt connected clients.
- Insufficient documentation for self-hosted deployment scenarios.

### Technical Context
- Interacts directly with the service binary, configuration files, and API endpoints.
- Manages API credentials, tokens, and user accounts.
- May run client apps alongside the service.

---

## Persona C — Third-Party Client Developer

**Name**: Alex  
**Role**: Open-source client application developer  
**Segment**: External developer / integrator

### Description
Alex builds or maintains a Firefly-III compatible client application (mobile, desktop, or web). Alex integrates against the published API contract and expects compatibility guarantees so that their app works without modification against this system for supported endpoints. Stability and predictability of API contracts are critical.

### Goals
- Develop a client app once and have it work against multiple Firefly-III compatible servers.
- Access clear, accurate API documentation and compatibility notes.
- Detect and handle API errors with well-defined, consistent error response formats.

### Pain Points
- Undocumented deviations from the Firefly-III API contract cause client failures.
- Inconsistent error response formats make robust error handling difficult.
- Breaking contract changes without versioning or migration guidance.

### Technical Context
- Uses REST API endpoints directly.
- Relies on JSON request/response schema compliance.
- Targets supported endpoints described in the compatibility matrix.

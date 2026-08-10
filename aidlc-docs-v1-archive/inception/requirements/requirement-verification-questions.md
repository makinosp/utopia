# Requirements Clarification Questions

Please answer the following questions to help clarify the project requirements.
Fill in your answer after each `[Answer]:` tag.

---

## Question 1
What would you like to build with this project?

A) Web application (browser-based frontend + backend)
B) API / backend service only
C) CLI tool / command-line application
D) Mobile application
E) Library / SDK / package
F) Data pipeline / batch processing system
X) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 2
What is the primary purpose / business goal of this project?
(Briefly describe what problem it solves or what value it delivers)

[Answer]: Build a personal finance / household budget management API that is partially compatible
with Firefly-III, so that existing client apps (e.g., Waterfly-III) can connect and operate
without modification. The goal is to provide a lightweight, self-hostable alternative to
Firefly-III while maintaining API compatibility with its ecosystem.

---

## Question 3
Who are the primary users or consumers of this system?

A) End users (general public / consumers)
B) Internal business users / employees
C) Developers / other systems via API
D) Automated systems / bots
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 4
What programming language or tech stack do you prefer?

A) TypeScript / Node.js
B) Python
C) Go
D) Rust
E) Java / Kotlin
F) No preference — let AI recommend based on requirements
X) Other (please describe after [Answer]: tag below)

[Answer]: D

---

## Question 5
What is the expected scale / deployment environment?

A) Personal / hobby project (local or small cloud deployment)
B) Small team / startup (moderate scale)
C) Enterprise / production-grade (high availability, scalability required)
D) Undecided at this point
X) Other (please describe after [Answer]: tag below)

[Answer]: A - starting as a personal/self-hosted project, but code quality and
API design should be suitable for public OSS use and others to self-host.

---

## Question 6
Do you have any specific non-functional requirements in mind?
(Select all that apply, or describe below)

A) High performance / low latency
B) Strong security and authentication
C) High availability and fault tolerance
D) Observability (logging, metrics, tracing)
E) None specific at this time
X) Other (please describe after [Answer]: tag below)

[Answer]: E

---

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)
B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)
B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with business logic, data transformations, serialization, or stateful components)
B) Partial — enforce PBT rules only for pure functions and serialization round-trips (suitable for projects with limited algorithmic complexity)
C) No — skip all PBT rules (suitable for simple CRUD applications, UI-only projects, or thin integration layers with no significant business logic)
X) Other (please describe after [Answer]: tag below)

[Answer]: B

---

# US-021 / US-022 — Authentication Feature Requirements

These questions are for implementing **US-021** (Obtain Personal Access Token) and **US-022** (Reject Unauthenticated Requests).

**Existing Auth Infrastructure**: Auth middleware with bearer validation (Argon2id + SHA256), token cache (positive/negative), audit logging, Prometheus metrics, `POST /api/v1/tokens`, `DELETE /api/v1/tokens/{id}`, `POST /api/v1/bootstrap/tokens`, and Firefly-III-compatible 401 error responses are already implemented. These questions focus on remaining gaps.

## Question A1
US-021 states: "Token issuance endpoint or mechanism exists and is documented." Authenticated `POST /api/v1/tokens` and bootstrap `POST /api/v1/bootstrap/tokens` exist. What additional work is needed?

A) The existing endpoints are sufficient — ensure OpenAPI spec and developer docs cover them

B) Add a token **list** endpoint (`GET /api/v1/tokens`) so users can see their active tokens

C) Add both a list endpoint AND developer documentation (e.g., a README section)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question A2
US-022 states: "All protected endpoints return HTTP 401 with a Firefly-III compatible error body." The auth middleware already returns 401 for all routes except `/api/v1/bootstrap/tokens` and `/metrics`. Any endpoints that should be explicitly excluded from auth?

A) No — current setup is correct

B) Yes — additional endpoints should be public (please describe)

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question A3
Should we add test coverage specifically for these stories?

A) Yes — add integration tests for token issuance, revocation, and 401 rejection

B) No — existing tests sufficiently cover auth scenarios

C) Yes — focused unit tests for parse_bearer edge cases and cache invalidation after revocation

X) Other (please describe after [Answer]: tag below)

[Answer]: A

## Question A4
Any rate-limiting or brute-force protection for token issuance endpoint?

A) No — token issuance is authenticated and admin-controlled

B) Yes — rate limit on `POST /api/v1/tokens` (e.g., max N tokens/hour/user)

C) Yes — rate limit on bootstrap token endpoint to prevent brute-force of bootstrap key

X) Other (please describe after [Answer]: tag below)

[Answer]: C - Enforce PBT rules only for pure functions and serialization round-trips, especially for monetary calculations, aggregation logic, and API payload round-trip consistency.

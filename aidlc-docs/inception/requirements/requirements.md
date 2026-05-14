# Requirements Document

## Intent Analysis Summary
- User request: Start AI-DLC workflow and define requirements for a Firefly-III partially compatible household finance API.
- Request type: New Project.
- Scope estimate: Single backend/API service with compatibility constraints.
- Complexity estimate: Moderate (API compatibility, security baseline, domain correctness for financial logic).

## Project Goal
Build a personal finance / household budget management API that is partially compatible with Firefly-III, so existing client apps (for example Waterfly-III) can connect and operate without modification for supported features.

## Functional Requirements
1. Provide an API-first backend service (no frontend required in initial scope).
2. Implement partial Firefly-III compatibility for prioritized endpoints and payload formats.
3. Preserve interoperability with existing Firefly-III ecosystem clients for supported flows.
4. Support core household bookkeeping use cases:
   - Account and balance representation
   - Transaction recording
   - Budget-related data handling
5. Expose stable, documented API contracts for clients and self-hosted deployments.

## Non-Functional Requirements
1. Primary language and stack preference: Rust.
2. Initial deployment target: personal/self-hosted scale.
3. Quality target: OSS-ready API design and implementation quality suitable for wider self-hosted adoption.
4. Security extension mode: Enabled (blocking security constraints).
5. Property-based testing extension mode: Partial enforcement.
   - Enforce PBT on pure functions and serialization round-trips.
   - Prioritize monetary calculations, aggregation logic, and API payload round-trip consistency.

## Constraints and Assumptions
1. Compatibility is explicitly partial, not full Firefly-III parity.
2. The exact compatibility matrix (supported resources/endpoints) will be finalized in subsequent stages.
3. "No specific NFR" response is interpreted as no additional NFR beyond selected extension constraints and stated quality target.

## Key Decisions Captured
1. Product direction: lightweight, self-hostable alternative focused on compatibility for client reuse.
2. Security posture: production-safe by design from the beginning.
3. Testing posture: targeted PBT where it yields highest value for financial correctness.

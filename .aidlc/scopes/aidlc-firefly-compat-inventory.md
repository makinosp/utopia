---
name: firefly-compat-inventory
depth: Minimal
keywords: []
description: Inventory of Firefly III-compatible API — implemented specs, upstream gaps, and prioritization
skeleton: on
review_cap: advisory
---

# firefly-compat-inventory scope

Analysis scope to inventory the current Firefly III-compatible API, map implemented specs against upstream Firefly III, and prioritize remaining gaps. No code implementation — deliverables are documentation and analysis only.

## Why these stages, why skip those

- **intent-capture** — Confirm inventory scope, upstream comparison baseline, and prioritization criteria (resolve IAE/UA)
- **reverse-engineering** — Scan openapi.yaml / handlers / modules / core / migrations to extract implemented specs (reduce CSU) — core
- **requirements-analysis** — Document the gap matrix against upstream Firefly III and the prioritization framework — core

Market research, feasibility, scope definition, team formation, mockups, design, implementation, verification, deployment, and operations are all out of scope (read-only inventory, R=LOW, VE=LOW). Limiting to three stages delivers the "current state → gaps → priorities" document at minimal cost.

## Membership

Initialization (3 stages) + intent-capture, reverse-engineering, requirements-analysis are EXECUTE. The remaining 26 stages are SKIP.

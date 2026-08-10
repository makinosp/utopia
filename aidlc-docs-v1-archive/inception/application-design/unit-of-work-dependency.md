# Unit of Work Dependency Matrix

## Dependency Strategy
- Enforce strict acyclic dependencies.
- Keep UOW-01 as the only shared upstream foundation.
- Prevent bidirectional coupling between domain units.
- Use shared interfaces instead of direct mutable cross-domain orchestration.

## Dependency Graph (Text)
UOW-01 Core Foundation -> UOW-02 Accounts -> UOW-03 Transactions

UOW-01 Core Foundation -> UOW-04 Budgets

UOW-01 Core Foundation -> UOW-05 Metadata

No reverse edges are allowed.

## Matrix

| From \ To | UOW-01 Core | UOW-02 Accounts | UOW-03 Transactions | UOW-04 Budgets | UOW-05 Metadata |
|---|---|---|---|---|---|
| UOW-01 Core | - | No | No | No | No |
| UOW-02 Accounts | Yes | - | No | No | No |
| UOW-03 Transactions | Yes | Yes (read/ownership checks only) | - | No | No |
| UOW-04 Budgets | Yes | No | No (consume projections/events only if needed) | - | No |
| UOW-05 Metadata | Yes | No | No | No | - |

## Allowed Integration Rules
- UOW-03 may read account existence and ownership contracts from UOW-02 but must not mutate account internals directly.
- UOW-04 must not call UOW-03 mutation flows directly; budget spending views should use stable projection contracts.
- UOW-05 remains read-focused and isolated from domain mutation chains.

## Delivery Sequence
1. UOW-01 Core Foundation
2. UOW-02 Accounts Module
3. UOW-03 Transactions Module
4. UOW-04 Budgets Module
5. UOW-05 Metadata Module

## Validation Results
- Dependency model is acyclic.
- Shared concerns are centralized.
- Domain correctness and API contract stability are prioritized.
- Unit boundaries are suitable for per-unit Functional Design and NFR stages.

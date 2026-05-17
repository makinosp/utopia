# Component Dependency

## Dependency Model
Architecture style: domain modules with internal layers and shared cross-cutting components.

## Dependency Rules
- API Handler depends on Auth, Domain Services, Compatibility Mapper, and Error Mapper.
- Domain Services depend on Repository traits and the PostgreSQL pool for explicit request transaction control.
- Domain Services may depend on Compatibility primitives only for normalization contracts, not transport DTOs.
- Compatibility Mapper depends on domain views and Firefly DTO definitions.
- Error Mapper depends on domain/auth/validation error types.
- Persistence implementation depends on PostgreSQL driver and schema layer.

## Dependency Matrix

| From | To | Type | Direction | Allowed |
|---|---|---|---|---|
| API Handler | Auth Service | Runtime | Inbound request gate | Yes |
| API Handler | Account/Transaction/Budget/Metadata Services | Runtime | Use-case dispatch | Yes |
| API Handler | Compatibility Mapper | Runtime | Response mapping | Yes |
| API Handler | Error Mapper | Runtime | Error translation | Yes |
| Domain Services | Repository Traits | Runtime | Data access abstraction | Yes |
| Domain Services | PostgreSQL Pool | Runtime | Explicit request transaction boundary | Yes |
| Domain Services | Other Domain Services | Runtime | Cross-domain usage | Prefer avoid |
| Compatibility Mapper | Domain Views | Compile-time | DTO transformation | Yes |
| Error Mapper | Domain/Auth Errors | Compile-time | Error transformation | Yes |
| Persistence Adapters | PostgreSQL | Runtime | Storage | Yes |

## Communication Patterns
- Internal pattern: direct in-process calls between components.
- Read path: handler -> service -> repository -> service -> mapper -> response.
- Write path: handler -> auth -> service -> repository chain -> mapper -> response.

## Data Flow Summary

## Request Flow
1. HTTP request enters API Handler.
2. Handler validates request schema and extracts bearer token.
3. Auth Service resolves principal.
4. Handler dispatches to target domain service.
5. Service executes business operation with repository interfaces and manages the transaction boundary directly.
6. Result view is mapped to Firefly-compatible payload.
7. Response is returned with standardized success or error schema.

## Error Flow
1. Error occurs at handler/service/repository boundary.
2. Error Mapper converts to Firefly-compatible error payload.
3. Handler returns mapped payload with appropriate HTTP status.

## Text Alternative for Visual Dependency Graph
- Hub: API Handler.
- Security entry: Auth Service.
- Use-case cores: Account Service, Transaction Service, Budget Service, Metadata Service.
- Shared cross-cutting: Compatibility Mapper and Error Mapper.
- Storage edge: Repository traits and PostgreSQL persistence adapters; request-scoped transactions are owned by services.

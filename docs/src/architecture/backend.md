# Backend (hexagonal)

The backend follows **ports-and-adapters** (hexagonal / clean) architecture. The
domain is pure and has no idea Postgres, Cedar, or gRPC exist; all I/O lives at
the edges behind traits. The whole model lives in `scylla-core`.

## Layers

```
scylla-core/src/
├── domain/          pure: entities, value objects, errors — no I/O
├── application/     use cases + ports (the traits the domain needs)
└── infrastructure/  adapters: Postgres, Cedar, Argon2, ChaCha, SMTP, OAuth…
```

Dependencies point **inward**: `infrastructure` depends on `application` depends on
`domain`. The domain never imports a library that touches the network or disk.

### Domain

Pure business types with no external dependencies:

- **Entities** (`domain/entities/`) — objects with identity and mutable state:
  `Pipeline`, `Job`, `App`, `Organization`, `Trigger`, …
- **Value objects** (`domain/value_objects/`) — immutable, *validated* wrappers
  built through fallible constructors that enforce an invariant: `PipelineName`,
  `NodeId`, `EnvKey`, `JobStatus`, `CronSpec`. If you hold one, it's valid.
- **Errors** (`domain/errors.rs`) — `DomainError` (`validation`, `business_rule`,
  `not_found`, …) returned as `DomainResult<T>`.

### Application

- **Use cases** — one struct per aggregate (`PipelineUseCases`, `JobUseCases`,
  `GrantUseCases`, …) holding `Arc<dyn Port>` dependencies and exposing async
  methods. They orchestrate; they don't do I/O directly.
- **Ports** — the traits a use case depends on: `PipelineRepository`,
  `PermissionService`, `HashService`, `CronSchedule`, `SecretCipher`, … Defined
  here, implemented in `infrastructure`.

### Infrastructure

Concrete **adapters** implementing the ports:

| Port (application) | Adapter (infrastructure) |
|--------------------|--------------------------|
| `PermissionService` | `CedarPermissionService` |
| `HashService` | `Argon2HashService` |
| `SecretCipher` | `ChaChaSecretCipher` (XChaCha20-Poly1305) |
| `CronSchedule` | `croner`-backed schedule |
| `…Repository` | `Pg…Repository` (sqlx / PostgreSQL) |
| mail / OAuth | `LettreMailer` / `GitHubOauthProvider` |

## Ports & adapters, concretely

A **port** is a trait describing something the domain needs from the outside; an
**adapter** is one implementation of it. Because use cases depend on the trait,
not the implementation, tests inject stubs and production injects the real thing —
the domain is unit-testable without a database or an authorization engine. The
`PermissionService` returns `DomainResult<()>` (never `bool`) so authorization is
[fail-closed by construction](./authorization.md).

## Crate composition

```
scylla-protocol ──► scylla-core ──► scylla-api ──► scylla-control-plane
   (proto +           (domain +      (gRPC          (binary: the
    bindings)          ports +        handlers +      composition root
                       adapters)       Services)       that wires it all)
```

`scylla-control-plane` is the single composition root: it reads config, builds the
concrete adapters, assembles the `Services`, and runs the servers. Swapping an
adapter (a different database, a different authz engine) is a change *there*, not
in the domain.

## Feature flags

`scylla-core` gates each domain area behind a Cargo feature, so a downstream crate
compiles in only what it uses. See `crates/scylla-core/Cargo.toml`.

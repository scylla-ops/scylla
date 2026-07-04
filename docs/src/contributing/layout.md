# Project layout

A map of the repository, so you know where a change belongs.

## Workspace crates

All Rust code is a Cargo workspace under `crates/`:

| Crate | Kind | Contents |
|-------|------|----------|
| `scylla-core` | lib | The domain model — entities, value objects, use cases, ports, adapters. The heart of the system. |
| `scylla-api` | lib | gRPC handlers, mappers, middleware, the `Services` composition, and the runners (`run_grpc`, webhook ingress). |
| `scylla-protocol` | lib | `.proto` definitions and generated Rust + TypeScript bindings. |
| `scylla-agent` | bin | The worker: connects, receives dispatches, executes the DAG, streams status/logs. |
| `scylla-control-plane` | bin | The composition root — reads config, wires adapters, runs the servers. |

Dependency direction: `protocol → core → api → control-plane`, with `agent`
depending on `core` + `protocol`. See [Backend](../architecture/backend.md).

## Inside `scylla-core`

```
scylla-core/src/
├── domain/
│   ├── entities/        Pipeline, Job, App, Organization, Trigger, …
│   ├── value_objects/   validated wrappers (NodeId, PipelineName, JobStatus, …)
│   └── errors.rs        DomainError / DomainResult
├── application/
│   └── <area>/          use cases + ports per area (pipeline, job, authz, secret, trigger, …)
├── infrastructure/
│   ├── persistence/postgres/   one Pg…Repository per aggregate (sqlx)
│   ├── services/               Cedar, Argon2, ChaCha, croner, lettre, GitHub OAuth
│   └── messaging/              in-memory agent registry + job-log stream
└── test_support/        shared fixtures & scenario builders for tests
```

## Top-level files

| Path | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest, shared deps, lints. |
| `justfile` | Task runner recipes (`dev` / `db` / `release`). |
| `docker-compose.yaml` / `Dockerfile` | The runnable stack + backend image. |
| `migrations/` | Versioned SQL, applied at boot via `sqlx::migrate!`. |
| `.sqlx/` | Committed offline query cache (compile without a live DB). |
| `crates/scylla-control-plane/config/` | Per-env TOML (`local`, `docker`, `prod`). |
| `examples/` | Sample pipeline definitions. |
| `apps/frontend/` | The React web UI (with its own `docs/`). |
| `docs/` | This book (mdBook). |
| `GLOSSARY.md` | Canonical term definitions. |

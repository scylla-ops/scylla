# Scylla Glossary

Reference for every domain-specific word used across Scylla's code, docs, and UI. Grouped by topic.

## Platform services

### `scylla-control-plane`
Single binary that boots the central brain: the gRPC API on `50051` (user APIs, app token exchange, and the agent worker stream). Composition root for `scylla-api`. Job dispatch and log fan-out are in-process — there is no message broker and no recorder. Config lives in `crates/scylla-control-plane/config/*.toml`.

### `scylla-api`
Library crate exposing the gRPC handlers (auth, organizations, projects, pipelines, jobs, apps, the worker stream, permissions) plus the `Services` composition struct and a `run_grpc` runner. Composed inside `scylla-control-plane`.

### `scylla-agent`
Worker binary installed on each pipeline-executing machine. Authenticates as its [App](#app) (`--app-id` / `--app-secret` exchanged for a bearer token), opens the control plane's `WorkerService` stream over `50051`, receives `JobDispatch` messages, walks the pipeline DAG in topological order (parallel within a level), spawns each node as a child process, and streams status + log events back on the same stream. Presence is simply the open stream — no heartbeats.

### `scylla-core`
Library crate containing the domain model (entities, value objects, use cases, ports). Not a service — it's imported by `scylla-api` and `scylla-control-plane`.

### `scylla-protocol`
Library crate holding shared `.proto` definitions and their generated Rust + TypeScript bindings. Both the backend and the frontend import these.

### `postgres`
Primary datastore. PostgreSQL 18 in the compose stack, listening on port `5432`. Schema is managed by versioned SQL files in `migrations/` applied via `sqlx::migrate!` at boot. The offline query cache lives in `.sqlx/` so Docker builds can compile without a live database.

## Domain entities

### Organization
Top-level tenant. Carries a `name`, optional `description`, and an `is_active` flag. Owns projects; users join via the `user_organization` table.

### Project
A unit of work inside an organization. Carries a `name`, optional `description`, `organization_id`, and `is_active` flag. Owns pipelines and jobs; users join via the `user_project` table.

### Pipeline
A directed acyclic graph (DAG) of **nodes** describing what to run. A pipeline is a blueprint — running it produces a **job**.

### Node (`PipelineNode`)
One step in a pipeline. Carries a `command`, `args`, and a list of `deps` (other node IDs it depends on). Nodes must form a DAG — cycles are rejected at creation time.

### Job
A single execution of a pipeline. Carries a `JobStatus` and a `JobNode` per pipeline node tracking per-node state and timestamps.

### JobNode
The runtime record for one pipeline node inside a job. Holds `NodeState`, `started_at`, `finished_at`.

### App
A machine principal owned by an organization (an agent / automation). Identified by an `AppId`; authenticates with a secret (stored hashed) that it exchanges for an app token, and acts under scoped grants — typically the `worker` role on its org. An App with an open worker stream is an online **agent**; "agent" now means an App running the worker binary.

### Session
An authenticated user session. Carries an opaque `token`, `user_id`, `created_at`, `expires_at`, and `last_active_at`. Created on login; the auth interceptor looks it up by token on each gRPC call and rejects expired sessions.

### User / Membership
A user account. Membership is modeled via the `user_organization` and `user_project` join tables — each row is just `(user_id, org_id)` or `(user_id, project_id)`. No role or permission is stored on the row itself (see [Permissions](#permissions)).

## States & status values

### `JobStatus`
- `Pending` — created, not yet dispatched.
- `Running` — an agent is executing at least one node.
- `Completed` — all nodes finished successfully.
- `Failed` — at least one node failed.
- `Cancelled` — user- or system-cancelled.
- `Orphaned` — running job lost its agent (e.g. agent disconnect without shutdown).

Terminal: `Completed`, `Failed`, `Cancelled`, `Orphaned`. Legal transitions: `Pending → Running | Cancelled`; `Running → Completed | Failed | Cancelled | Orphaned`.

### `NodeState`
Per-node execution state: `Pending`, `Running`, `Completed`, `Failed`, `Cancelled`. Terminal: `Completed`, `Failed`, `Cancelled`. (Unlike `JobStatus`, there is no `Orphaned` node state.)

### Terminal state
A state from which no transition is possible. A job or node in a terminal state is done.

## Permissions

Scylla has no role entity. Every check is a direct `(subject, policy)` lookup — subjects are users, policies are triples. Memberships (`user_organization`, `user_project`) only record which orgs/projects a user belongs to; they carry no permission info.

### Policy
A triple `(scope, resource, act)` describing one permitted action. Built per call site in `scylla-core/src/domain/value_objects/permission/policy.rs` (e.g. `pipeline::run(pipeline_id)`).

### Scope
Where a policy applies: `System`, `Org(id)`, `Project(id)`, `User(id)`, or `All`. Serialized as `system`, `org/<id>`, `project/<id>`, `user/<id>`, `*`.

### Resource
What the policy governs. One of `User`, `Pipeline`, `Job`, `Project`, `Organization`, `Agent`, or `All`. Each variant wraps a `Target::All` (any instance) or `Target::Single(id)`.

### Target
Resource granularity: `All` (the whole kind, e.g. all pipelines) or `Single(id)` (one specific entity).

### Act
The verb: `Create`, `Read`, `Write`, `Delete`, `Execute`, or `All`. Serialized as `create`, `read`, `write`, `delete`, `execute`, `*`.

### `PermissionService`
Port (`application/ports/services/permission_service.rs`) used by every use case to check policies. The production adapter is Casbin-backed.

### Casbin
External authorization engine used by the permission service. Policies are persisted in PostgreSQL through `sqlx-adapter` (the `casbin_rule` table is created by the dedicated migration).

### Absolute policy
`Policy::absolute()` = `(Scope::All, Resource::All, Act::All)`. Granted to the bootstrap user so the first account can do anything.

### Bootstrap user
The initial `admin` account created on first control-plane startup (configured under `[bootstrap]` in the control-plane config). Default credentials in local dev: `admin` / `admin123`. Gets the absolute policy.

### `CallerContext`
Identity-aware principal threaded through the permission layer. Variants: `User(UserId)`, `App(AppId)` (a machine principal / agent), `Service(ServiceIdentity)`, `Anonymous`. Each maps to a Cedar entity (`Scylla::User::"…"`, `Scylla::App::"…"`, `Scylla::Service::"…"`); the Cedar `PermissionService` resolves the caller's roles, scoped grants, and tenancy ancestry on each check.

## Pipeline execution concepts

### DAG (Directed Acyclic Graph)
The shape a pipeline's nodes must form. Validated at pipeline creation via cycle detection.

### Topological order
A linear ordering of nodes consistent with dependencies (each node appears after its deps). Scylla uses **Kahn's algorithm** over a `BTreeSet` for deterministic output.

### Kahn's algorithm
Iterative topological sort: repeatedly pop nodes with in-degree zero, decrement in-degree of their dependents. Used here for both validation (cycle detection) and ordering.

### Adjacency (map)
Forward edge map from a node to its dependents. Computed on demand from `nodes[].deps`.

### Dependency (`deps`)
A prerequisite node ID. A node only becomes runnable once all its deps are in a successful terminal state.

## Agent presence

### Connected (online)
An agent is connected when its [App](#app) holds an open `WorkerService` stream to the control plane. Presence is tracked in memory (the worker registry), not persisted — there are no heartbeats. App listings expose this as a `connected` flag, and `RunPipeline` only dispatches to a connected, authorized worker.

## Networking & protocol

### gRPC
Primary transport between API and internal services. Defined in `.proto` files in `crates/scylla-protocol/proto/`.

### gRPC-Web
Browser-compatible variant of gRPC spoken by the frontend via `@protobuf-ts/grpcweb-transport`. Served by the control plane through `tonic-web`.

### Tonic
Rust gRPC server/client framework used by all backend services.

### Prost
Protobuf code generator used by Tonic. Converts `.proto` → Rust structs.

### `protobuf-ts`
TypeScript protobuf toolchain used by the frontend to generate clients from `.proto`.

### Auth interceptor
Async Tonic interceptor (`crates/scylla-api/src/grpc/middleware/auth_interceptor.rs`). Reads the `authorization: Bearer <token>` metadata and resolves it to a principal — a user session (`SessionRepository`) or, failing that, an app token (`AppTokenRepository`). Rejects expired or unknown tokens with `Unauthenticated` and attaches an `AuthContext { caller }` (`CallerContext::User` or `CallerContext::App`) to the request extensions.

## Identifiers

### Entity IDs
All domain IDs (`UserId`, `OrganizationId`, `ProjectId`, `PipelineId`, `JobId`, `JobLogId`, `SessionId`, `AppId`, `AppTokenId`, `UserOrganizationId`, `UserProjectId`) are opaque string newtypes generated as lowercased ULIDs via `::generate()`. They also accept external strings via `::new(...)`.

### `NodeId`
Caller-supplied ID for each pipeline node. Must be unique within its pipeline. Validated as a lowercase ASCII alphanumeric string plus `-` / `_`, max 128 chars.

## Infra & dev

### `docker-compose.yaml`
Defines the backend stack (`postgres`, `scylla-control-plane`, `scylla-frontend`). Agents are installed out-of-band (see [`scylla-agent`](#scylla-agent)), not in this stack.

### `justfile`
Task runner recipes: `just up`, `just down`, `just logs`, `just push-all`, etc.

### `config/*.toml`
Per-environment config for `scylla-control-plane` (under `crates/scylla-control-plane/config/`): `local.toml` (host-native dev), `docker.toml` (compose), `prod.toml` (production).

### `VITE_API_URL`
Frontend env var pointing the gRPC-Web client at the API (default: same origin). Set in `apps/frontend/.env.local` to override.

### Lingui
i18n framework used by the frontend. `pnpm extract` / `pnpm compile` manage message catalogs under `apps/frontend/src/locales/`.

## Architecture terms

Scylla's layout inside `scylla-core`:

- `src/domain/` — entities, value objects, errors (pure, no I/O).
- `src/application/ports/` — trait definitions (`repositories/`, `services/`).
- `src/application/use_cases/` — orchestrators that consume ports.
- `src/infrastructure/` — concrete adapters (PostgreSQL via sqlx, Casbin, Argon2).

### Port
A trait in `application/ports/` describing something the domain needs from the outside world (persistence, hashing, permission checks). Examples: `PipelineRepository`, `HashService`, `PermissionService`.

### Adapter
A concrete implementation of a port. All current adapters live under `scylla-core/src/infrastructure/` — `persistence/postgres/*` for repos, `services/casbin_permission_service.rs`, `services/argon2_hash_service.rs`.

### Use case
A struct in `application/use_cases/*.rs` grouping operations on one aggregate (e.g. `PipelineUseCases`, `JobUseCases`). Holds `Arc<dyn Port>` fields and exposes async methods. gRPC handlers in `scylla-api` call these.

### Entity
A domain object with an identity and mutable state (e.g. `Pipeline`, `Job`, `App`, `Organization`). Defined in `domain/entities/`.

### Value object
An immutable, validated wrapper in `domain/value_objects/` (e.g. `PipelineName`, `Hostname`, `NodeId`, `JobStatus`). Built via a fallible constructor that enforces the invariant.

### Repository
A port describing persistence for one aggregate (`PipelineRepository`, `JobRepository`, ...). Trait lives in `application/ports/repositories/`; the PostgreSQL implementation lives in `infrastructure/persistence/postgres/` (one `Pg…Repository` per aggregate, queries via `sqlx::query!` / `query_as!`).

### Domain error
`DomainError` from `domain/errors.rs` with variants like `validation`, `business_rule`, `not_found`. Returned as `DomainResult<T>` from core and mapped to gRPC statuses by handlers in `scylla-api`.

### Feature flags
`scylla-core` gates each domain (users, pipelines, jobs, agents, ...) behind a Cargo feature, so downstream crates pull only what they need. See `crates/scylla-core/Cargo.toml`.

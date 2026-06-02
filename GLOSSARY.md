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
A machine principal owned by an organization (an agent / automation). Identified by an `AppId`; authenticates with a secret (stored hashed) that it exchanges for an app token, and acts under scoped grants — typically the `organization-agent` role on its org. An App with an open agent stream is an online **agent**; "agent" now means an App running the agent binary.

### Session
An authenticated user session. Carries an opaque `token`, `user_id`, `created_at`, `expires_at`, and `last_active_at`. Created on login; the auth interceptor looks it up by token on each gRPC call and rejects expired sessions.

### User / Membership
A user account. Membership is modeled via the `user_organization` and `user_project` join tables — each row is just `(user_id, org_id)` or `(user_id, project_id)`. No role or permission is stored on the row itself (see [Authorization](#authorization)).

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

## Authorization

The vocabulary below is the **single source of truth** for authorization words —
one word per concept, used identically in code, proto, DB, and UI. The model is
mid-refactor toward dynamic RBAC; see [`docs/authz-refactor.md`](docs/authz-refactor.md)
for the target model, phased plan, and status.

**The seven words:** **Permission** (atomic capability) · **Role** (bundle of
permissions) · **Scope** (System/Organization/Project) · **Grant** (P holds X in
scope S) · **Principal** (a grant-holding User\|App) · **Caller** (the request
identity) · **Policy** (advanced Cedar rule). Plus **Resource** (the entity an
action targets). "action" is retired from the domain vocabulary — it survives
only as Cedar's wire term inside the infra adapter.

### Role naming convention
Builtin role names follow **`<scope>-<role>`**, kebab-case, with scope ∈ {`system`,
`organization`, `project`}. Single source of truth: the `*_ROLE` constants in
`application/authz/grant.rs`.

**Unified model:** there is ONE authorization mechanism — the **grant** (`grants`), a `(principal, role, scope)` triple linked as a Cedar role-template instance. A "global role" is just a grant on the **System** scope (the tenancy root: `Organization`/`User` are `in [System]`, so a System grant reaches everything). There is no separate `user_roles` table and no `RoleService` — `GrantService.CreateGrant(user, role, GRANT_SCOPE_SYSTEM)` replaces the old `AssignRole`.

Canonical roles (all stored in `grants`):

| Role | Scope | Cedar template | Confers |
|------|-------|----------------|---------|
| `system-admin` | System | full-control | everything, every scope (System is the root) |
| `organization-admin` | Organization | full-control | all actions on the org + everything beneath |
| `project-admin` | Project | full-control | all actions on the project + everything beneath |
| `organization-agent` | Organization | restricted agent | `readPipeline`/`executeJob`/`writeJobStatus`/`writeJobLog` within the org |
| `project-agent` | Project | restricted agent | same, within a project |

**Implicit tiers (NOT named roles):** `system-member` (a plain user with no grant), `organization-member` / `project-member` (membership via the `user_organization` / `user_project` tables, granted read/operate access through ABAC policies). They follow the same naming vocabulary but are realized as membership/ABAC, not stored grants.

### Permission
The atomic capability — a verb on a resource type, e.g. `runPipeline` on a
pipeline. Closed, code-owned catalog: the `Permission` enum
(`domain/value_objects/permission/`), one variant per real enforced capability.
`Permission::key()` is its canonical id (`"runPipeline"`) — the value a role
stores and the Cedar `Action::"…"` eid. A permission cannot be created at runtime
(only the code that enforces it gives it meaning); **roles** are the dynamic part.

### Role
A named bundle of permissions bound to a scope kind, stored in the `roles` +
`role_permissions` tables. Builtin roles (`system-admin`, `organization-admin`,
`project-admin`, `organization-agent`, `project-agent`) are global and seeded on
first boot; custom roles are owned by an Org (tenant-isolated). A grant of a role
confers all its permissions within the grant's scope. The live Cedar policy set
is **generated** from these rows (a full-control role — permission `*` — maps to
the unconstrained-action body; any other role lists its permission keys), so
editing a role's permissions changes authorization on the next reload.

### Scope
The level a grant/role binds to: `System` (tenancy root), `Organization(id)`, or
`Project(id)`. `Scope` carries the id; `ScopeKind` is its id-free discriminant.
Because every entity is `in` its scope ancestors (Project `in` Org `in` System),
a grant at a scope reaches everything beneath it.

### Grant
"Principal P holds {a role | a single permission} within scope S." The one
authorization mechanism — stored in `grants`, linked into Cedar on reload. A
direct **permission** grant (e.g. Alice `runPipeline` in Org A) is additive to
P's role-derived permissions.

### Principal
A grant-holding actor: a human `User` or a machine `App`. Maps to the
`(principal_kind, principal_id)` columns of `grants` and to the Cedar
`?principal` slot. Distinct from **Caller**: a Caller may also be a Service or
Anonymous, which cannot hold grants.

### Policy
**Only** an advanced Cedar escape-hatch rule (a `permit`/`forbid` in the
`cedar_policies` table), layered on top of the role/grant-derived policy set for
cases RBAC can't express. Not a synonym for permission or grant.

### `PermissionService`
Port (`application/authz/service.rs`) used by every use case to check a
permission: `check(caller, Permission) -> DomainResult<()>` (fail-closed — never
a bool). The production adapter is **Cedar**-backed (`CedarPermissionService`):
it builds the principal/resource entities, snapshots the live policy set, asks
Cedar, and records an audit-log row.

### Cedar
The authorization engine (AWS Cedar). Scylla generates its policy set from the
RBAC model (roles + grants → `permit` policies), keeps a static ABAC base
(`policies.cedar`: membership, self-read) and the admin-defined `cedar_policies`.
Resolves the scope hierarchy via `in` and yields the allow/deny decision +
diagnostics.

### Bootstrap user
The initial `admin` account created on first control-plane startup (configured under `[bootstrap]` in the control-plane config). Default credentials in local dev: `admin` / `admin123`. Gets a `system-admin` grant on the `System` scope (full control over every scope, since System is the tenancy root).

### Caller (`CallerContext`)
The identity that made a request, threaded through the authorization layer. Variants: `User(UserId)`, `App(AppId)` (a machine principal / agent), `Service(ServiceIdentity)` (sealed internal identity), `Anonymous`. A Caller that is a `User` or `App` is also a **Principal** (can hold grants); `Service`/`Anonymous` cannot. Each maps to a Cedar entity (`Scylla::User::"…"`, `Scylla::App::"…"`, `Scylla::Service::"…"`); the Cedar `PermissionService` resolves the caller's roles, scoped grants, and tenancy ancestry on each check.

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

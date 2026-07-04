# Glossary

Every Scylla-specific term, grouped by topic. This is the book's canonical
reference; where a concept has a whole chapter, the entry links to it.

## Platform services

**`scylla-control-plane`** — The central binary: the gRPC API on `50051` (user
APIs, app-token exchange, and the agent worker stream) plus in-process job
dispatch and log fan-out. The composition root — no message broker, no separate
recorder.

**`scylla-api`** — Library crate with the gRPC handlers, the `Services`
composition struct, and the `run_grpc` / webhook runners. Composed inside the
control plane.

**`scylla-core`** — Library crate holding the domain model (entities, value
objects, use cases, ports) and its adapters. Imported by `scylla-api` and the
control plane.

**`scylla-protocol`** — Library crate with the shared `.proto` files and their
generated Rust + TypeScript bindings.

**`scylla-agent`** — Worker binary installed per machine. Authenticates as its App,
opens the worker stream, executes dispatched jobs, and streams status + logs back.

**`postgres`** — Primary datastore (PostgreSQL 18). Schema managed by
`migrations/*.sql` applied via `sqlx::migrate!` at boot; offline query cache in
`.sqlx/`.

## Domain entities

**Organization** — Top-level tenant. Owns projects; users join via
`user_organization`.

**Project** — A unit of work inside an org. Owns pipelines, jobs, and secrets;
users join via `user_project`.

**Pipeline** — A directed acyclic graph (DAG) of nodes describing what to run. A
blueprint; running it produces a job. See [Writing pipelines](../using/pipelines.md).

**Node (`PipelineNode`)** — One step in a pipeline: an `id`, `deps`, a `step`
(`exec` or `script`), an optional `working_dir`, and an `env` overlay. Nodes must
form a DAG.

**Step** — What a node runs: an **`exec`** (command + literal argv, no shell) or a
**`script`** (multi-line shell script, `sh` or `bash`, fail-fast).

**Job** — A single execution of a pipeline. Carries a `JobStatus` and a `JobNode`
per pipeline node.

**JobNode** — The runtime record for one node inside a job: its `NodeState` plus
`started_at` / `finished_at`.

**App** — A machine principal owned by an org (an agent / automation). Authenticates
with a hashed secret exchanged for an app token, and acts under scoped grants
(typically `organization-agent`). An App with an open worker stream is an online
agent.

**Session** — An authenticated user session: an opaque token with `user_id`,
`created_at`, `expires_at`, `last_active_at`. Resolved by the auth interceptor on
each call.

**User / Membership** — A user account. Membership is a plain `(user_id, org_id)`
or `(user_id, project_id)` row — no role stored on it (see Authorization).

## States & status

**`JobStatus`** — `Pending` → `Running` → terminal `Completed` · `Failed` ·
`Cancelled` · `Orphaned`. `Orphaned` = a running job lost its agent.

**`NodeState`** — Per-node: `Pending`, `Running`, `Completed`, `Failed`,
`Cancelled` (no `Orphaned`).

**Terminal state** — A state with no outgoing transition; the job or node is done.

## Authorization

See [the authorization model](../architecture/authorization.md) for the full
treatment.

**Permission** — An atomic capability (a verb on a resource type, e.g.
`runPipeline`). A closed, code-owned catalog; cannot be created at runtime.

**Role** — A named bundle of permissions bound to a scope kind. Builtins are seeded;
custom roles are org-owned. The live Cedar policy set is generated from roles.

**Scope** — The level a grant binds to: `System` (tenancy root), `Organization(id)`,
or `Project(id)`. A grant reaches everything in its subtree.

**Grant** — *Principal P holds {a role | a permission} within scope S* — the one
authorization mechanism, stored in `grants`.

**Principal** — A grant-holding actor: a `User` or an `App`.

**Caller (`CallerContext`)** — The request identity: `User`, `App`, `Service`, or
`Anonymous`. Users and Apps are also Principals; Service/Anonymous can't hold
grants.

**Policy** — An advanced Cedar escape-hatch rule in `cedar_policies`; **not** a
synonym for permission or grant.

**Cedar** — The authorization engine (AWS Cedar). Scylla generates its policy set
from roles + grants, over a static ABAC base, and asks it for each decision.

**`PermissionService`** — The port every use case checks:
`check(caller, Permission) -> DomainResult<()>` — fail-closed, never a bool.
Production adapter is `CedarPermissionService`.

**Builtin roles** — `system-admin`, `organization-admin`, `project-admin`
(full control at their scope), `organization-agent`, `project-agent` (restricted:
read pipeline, execute job, write status/log). Names follow `<scope>-<role>`.

**Bootstrap user** — The initial `admin` account created on first boot from
`[bootstrap]`; gets a `system-admin` grant on `System`.

## Pipeline execution

See [Pipeline execution](../architecture/execution.md).

**DAG** — The acyclic shape a pipeline's nodes must form; validated at creation.

**Topological order** — A linear ordering consistent with dependencies. Scylla uses
Kahn's algorithm over a `BTreeSet` for deterministic output.

**Kahn's algorithm** — Iterative topological sort (pop in-degree-zero nodes,
decrement dependents). Used for both cycle detection and ordering.

**Dependency (`deps`)** — A prerequisite node id; a node runs only once all its
deps reach a successful terminal state.

**Workspace** — The per-job directory `<workspace-root>/<job-id>` shared by all of a
job's nodes, so artifacts flow downstream. Removed when the job ends unless
`--keep-workspace`.

## Agent presence

**Connected (online)** — An agent is connected when its App holds an open
`AgentService` worker stream. Presence is in-memory, no heartbeats; `RunPipeline`
only dispatches to a connected, authorized worker.

## Networking & protocol

**gRPC** — Primary transport, defined in `crates/scylla-protocol/proto/`.

**gRPC-Web** — Browser-compatible variant the frontend speaks via
`@protobuf-ts/grpcweb-transport`, served through `tonic-web`.

**Tonic / Prost** — The Rust gRPC framework and protobuf code generator.

**Auth interceptor** — Async Tonic interceptor that resolves the `Bearer` token to a
user session or app token and attaches the `CallerContext`.

## Identifiers

**Entity IDs** — Domain ids (`UserId`, `OrganizationId`, `ProjectId`, `PipelineId`,
`JobId`, `AppId`, …) are lowercased ULID string newtypes via `::generate()`.

**`NodeId`** — Caller-supplied, unique within its pipeline: lowercase ASCII
alphanumeric plus `-`/`_`, ≤ 128 chars.

**`EnvKey`** — A node env var name: `^[A-Za-z_][A-Za-z0-9_]*$`, excluding the
reserved `SCYLLA_` prefix (the agent owns that namespace).

## Architecture terms

**Port** — A trait in `application/` describing something the domain needs from
outside (persistence, hashing, authz).

**Adapter** — A concrete implementation of a port, under `infrastructure/`
(`Pg…Repository`, `CedarPermissionService`, `Argon2HashService`, …).

**Use case** — A struct in `application/` grouping operations on one aggregate,
holding `Arc<dyn Port>` fields; called by gRPC handlers.

**Entity / Value object** — An identity-bearing domain object vs. an immutable,
validated wrapper. See [Backend](../architecture/backend.md).

**Domain error** — `DomainError` (`validation`, `business_rule`, `not_found`, …),
returned as `DomainResult<T>` and mapped to gRPC statuses by handlers.

**Feature flags** — `scylla-core` gates each domain area behind a Cargo feature so
downstream crates pull only what they need.

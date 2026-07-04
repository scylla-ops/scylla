# System overview

Scylla has a deliberately small moving-parts count: **one control plane, many
agents, a persistent stream between them, and no message broker.**

## Components

| Component | Crate / image | Role |
|-----------|---------------|------|
| Control plane | `scylla-control-plane` | The central brain: gRPC API, in-process job dispatch, webhook ingress. |
| API library | `scylla-api` | gRPC handlers + composition, run inside the control plane. |
| Domain | `scylla-core` | Entities, use cases, ports, adapters. |
| Protocol | `scylla-protocol` | Shared `.proto` + generated Rust & TypeScript. |
| Agent | `scylla-agent` | Worker that executes pipeline nodes. |
| Datastore | `postgres` | The single source of persistent truth. |
| Frontend | `scylla-frontend` | The web UI. |

The control plane is a **single binary** — no separate scheduler, dispatcher,
or recorder process; job dispatch and log fan-out happen in-process.
`scylla-api` is a *library* composed inside it, not its own service.

## The worker stream

Agents don't poll. Each authenticates as its
[App](../using/concepts.md#apps--agents), then opens a long-lived gRPC stream:

```
                 ┌───────────────────────── control plane (50051) ─────────────────────────┐
                 │   gRPC API   ·   in-process dispatch   ·   log fan-out   ·   Cedar authz  │
                 └───▲───────────────────────────▲───────────────────────────────▲──────────┘
   gRPC-Web         │                            │  worker stream                │  gRPC
  ┌──────────┐      │              ┌─────────────┴─────────────┐                 │
  │ web UI    │─────┘              │ JobDispatch  ▼   ▲  status+logs              │
  └──────────┘                     │            agent (App)     │           ┌─────┴─────┐
                                   └────────────────────────────┘           │  postgres │
                                                                            └───────────┘
```

The control plane pushes `JobDispatch` messages down; the agent streams node
status and log lines back up. **Presence is the open stream** — no heartbeats;
if it closes, the agent is gone and its running job becomes `Orphaned`.

## Request lifecycle

A typical write, end to end:

1. The browser calls the gRPC-Web API on `50051`.
2. The **auth interceptor** resolves the bearer token to a caller (user
   session or app token) and attaches it to the request.
3. A gRPC **handler** in `scylla-api` translates the request and calls a
   **use case** in `scylla-core`.
4. The use case checks a **permission** (Cedar, fail-closed), then drives
   **repositories** and **services** (ports) to do the work.
5. For a pipeline run, the use case picks a connected, authorized agent and
   dispatches the job over its worker stream; the agent streams logs back,
   which the control plane fans out to any UI tailing them and persists.

The layering behind steps 3–4 is [the hexagonal backend](./backend.md); the
authorization in step 4 is [the authorization model](./authorization.md); the
agent side of step 5 is [pipeline execution](./execution.md).

## Boot & shutdown

The control plane is a single composition root: it builds the shared
services, starts the gRPC server (plus a concurrent webhook HTTP server if
`[webhook]` is configured), and runs until `Ctrl+C` / `SIGTERM`, which cancels
one root token that shuts both servers down and closes the database pool.

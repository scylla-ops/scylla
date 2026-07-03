# Core concepts

A short tour of the words you need before using Scylla. Each has a precise
definition in the [Glossary](../reference/glossary.md); this page is the mental
model.

## Organizations & projects

An **Organization** is the top-level tenant — the boundary that owns everything
and that users belong to. Inside an org live **Projects**, each a unit of work
that owns pipelines, jobs, and secrets. Users join an org or a project through
membership; what they can *do* is decided by [access control](./access.md).

```
Organization
└── Project
    ├── Pipelines
    ├── Jobs
    └── Secrets
```

## Pipelines & nodes

A **Pipeline** is a blueprint: a directed acyclic graph (**DAG**) of **nodes**.
Each node runs one thing (a shell script or a direct command) and may declare
**dependencies** on other nodes. A node runs only after every dependency has
finished successfully; nodes with no unmet dependency run in parallel.

Because it is a *acyclic* graph, cycles are illegal — Scylla rejects them when
the pipeline is created. A pipeline is inert on its own; it describes work but
does not perform it.

## Jobs

Running a pipeline produces a **Job** — one concrete execution. The job tracks an
overall **status** and, for every pipeline node, a **JobNode** holding that
node's own state and start/finish timestamps. So a pipeline is the recipe and a
job is one time you cooked it.

Job status moves through a small state machine:

`Pending` → `Running` → one of `Completed` · `Failed` · `Cancelled` · `Orphaned`

The last four are terminal. `Orphaned` is special — it means a running job lost
its agent (the worker disconnected without reporting a result).

## Apps & agents

An **App** is a *machine principal* — a non-human identity owned by an
organization, with its own credentials. A **Agent** is an App running the
`scylla-agent` binary and connected to the control plane, ready to execute jobs.

The distinction matters: you create the App in the UI (which mints its
credentials), then launch an agent process with those credentials. An App with a
live connection shows as **connected**, and only connected agents receive work.
There are no heartbeats — presence *is* the open connection. See
[Running an agent](./agents.md).

## Users & access

People sign in as **Users**. Membership puts a user inside an org or project;
**roles** and **grants** decide their capabilities. A *grant* says "this
principal holds this role within this scope" — for example, org-admin over one
organization. The full model, including how machine agents get their scoped
permissions, is covered in [Users, orgs & access](./access.md) and, in depth, in
[the authorization model](../architecture/authorization.md).

## How it fits together

```
User ──creates──► Pipeline ──run──► Job ──dispatched to──► Agent (an App)
                    │                 │                        │
              graph of nodes    per-node state          executes each node,
                                                          streams logs back
```

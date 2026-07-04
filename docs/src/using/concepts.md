# Core concepts

The mental model behind Scylla's vocabulary; precise definitions live in the
[Glossary](../reference/glossary.md).

## Organizations & projects

An **Organization** is the top-level tenant — it owns everything, and users
belong to it. Inside live **Projects**, each owning pipelines, jobs, and
secrets. Users join an org or project through membership; what they can *do*
is decided by [access control](./access.md).

## Pipelines & nodes

A **Pipeline** is a blueprint: a directed acyclic graph (**DAG**) of **nodes**.
Each node runs one thing (a shell script or a direct command) and may declare
**dependencies** on other nodes. A node runs only after every dependency has
finished successfully; nodes with no unmet dependency run in parallel. Cycles
are rejected when the pipeline is created. A pipeline is inert on its own — it
describes work but does not perform it.

## Jobs

Running a pipeline produces a **Job** — one concrete execution. The job tracks
an overall **status** and, per pipeline node, a **JobNode** holding that node's
state and start/finish timestamps.

`Pending` → `Running` → one of `Completed` · `Failed` · `Cancelled` · `Orphaned`

The last four are terminal. `Orphaned` means a running job lost its agent (the
worker disconnected without reporting a result).

## Apps & agents

An **App** is a *machine principal* — a non-human identity owned by an
organization, with its own credentials. An **agent** is an App running the
`scylla-agent` binary, connected to the control plane. You create the App in
the UI (which mints its credentials), then launch an agent process with them.
Presence *is* the open connection — there are no heartbeats — and only
**connected** agents receive work. See [Running an agent](./agents.md).

## Users & access

People sign in as **Users**. Membership puts a user inside an org or project;
**roles** and **grants** decide their capabilities. A *grant* says "this
principal holds this role within this scope" — for example, org-admin over one
organization. See [Users, orgs & access](./access.md) and, in depth,
[the authorization model](../architecture/authorization.md).

## How it fits together

```
User ──creates──► Pipeline ──run──► Job ──dispatched to──► Agent (an App)
                    │                 │                        │
              graph of nodes    per-node state          executes each node,
                                                          streams logs back
```

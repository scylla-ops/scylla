# Writing pipelines

A pipeline is a directed acyclic graph (DAG) of **nodes**; the edges are
**dependencies**.

## Anatomy of a pipeline

| Field        | Meaning                                  |
|--------------|------------------------------------------|
| `name`       | Human label, unique within the project.  |
| `project_id` | The project that owns it.                |
| `nodes`      | One or more nodes forming the DAG.       |

A pipeline must have **at least one node**. Editing a pipeline replaces its
whole node set at once, re-validated as a unit.

## Nodes

| Field         | Required | Meaning                                                            |
|---------------|----------|--------------------------------------------------------------------|
| `id`          | yes      | Unique node identifier within the pipeline (see rules below).      |
| `deps`        | no       | IDs of nodes that must finish successfully before this one starts. |
| `step`        | yes      | What the node runs — a **script** or an **exec** (see below).      |
| `working_dir` | no       | Directory to run in, relative to the job workspace. Empty = root.  |
| `env`         | no       | Node-scoped environment variables (literals or secret references). |

A node `id` must be **lowercase** ASCII alphanumeric plus `-` and `_`, at most
128 characters, and unique within the pipeline (e.g. `build`, `run-tests`,
`deploy_prod`).

## Steps: `script` vs `exec`

Every node runs exactly one of two step kinds:

- **`script`** — a (possibly multi-line) shell script with **fail-fast**
  semantics: a failing line aborts with a non-zero exit. Shells: `sh` (POSIX
  `/bin/sh`, always present in the agent image — the default) or `bash`
  (requires bash in the agent's base image). The ergonomic choice for
  pipelines, loops, and multiple commands.
- **`exec`** — a command resolved via `PATH` plus a literal argument vector.
  No shell, so no word-splitting, globbing, or injection surface. Reach for
  this for determinism or when handling untrusted input.

## Environment variables

Each node carries its own `env` overlay. A value is either an inline
**literal** or a reference to a project [secret](./secrets.md), resolved and
decrypted control-plane-side at dispatch — the agent only ever sees the
resolved value.

Key rules:

- Must match `^[A-Za-z_][A-Za-z0-9_]*$`.
- May **not** start with the reserved prefix `SCYLLA_` — the agent injects its
  own context variables (`SCYLLA_WORKSPACE`, `SCYLLA_JOB_ID`, …) under that
  prefix authoritatively, so your keys can't shadow them.

## The DAG rules

The node set is validated when a pipeline is created or updated. It is
rejected if:

- the pipeline has **no nodes**;
- two nodes share an `id` (**duplicate node ID**);
- a node depends on an `id` that doesn't exist (**invalid dependency**);
- a node lists itself in its own `deps` (**self-dependency**);
- a node lists the same dependency twice (**duplicate dependency**);
- the dependencies form a **cycle**.

Cycle detection uses Kahn's algorithm; ordering is covered in
[Pipeline execution](../architecture/execution.md).

## Worked example

A classic fan-out / fan-in: `setup` runs first; `build`, `test`, and `lint`
each depend only on `setup`, so they run in parallel once it succeeds;
`report` depends on all three and runs after they finish.

```
        ┌──► build ─┐
setup ──┼──► test  ─┼──► report
        └──► lint  ─┘
```

## Workspaces

All nodes of a job share one workspace directory (`<root>/<job_id>` on the
agent), so artifacts a node writes are visible downstream — `build` can leave
a binary that `report` reads. The workspace is created when the job starts and
removed when it ends (unless the agent is told to keep it for debugging). Use
`working_dir` to run a node in a subdirectory of it.

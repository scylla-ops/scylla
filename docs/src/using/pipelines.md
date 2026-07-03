# Writing pipelines

A pipeline is a directed acyclic graph (DAG) of **nodes**. Each node runs one
step; the edges between nodes are **dependencies**. This chapter covers how to
shape that graph and what each node can do.

## Anatomy of a pipeline

A pipeline has three things:

| Field        | Meaning                                            |
|--------------|----------------------------------------------------|
| `name`       | Human label, unique within the project.            |
| `project_id` | The project that owns it.                           |
| `nodes`      | One or more nodes forming the DAG.                  |

A pipeline must have **at least one node**. Editing a pipeline replaces its whole
node set at once, re-validated as a unit.

## Nodes

Each node describes one unit of execution:

| Field         | Required | Meaning                                                            |
|---------------|----------|--------------------------------------------------------------------|
| `id`          | yes      | Unique node identifier within the pipeline (see rules below).      |
| `deps`        | no       | IDs of nodes that must finish successfully before this one starts. |
| `step`        | yes      | What the node runs — a **script** or an **exec** (see below).      |
| `working_dir` | no       | Directory to run in, relative to the job workspace. Empty = root.  |
| `env`         | no       | Node-scoped environment variables (literals or secret references). |

### Node IDs

A node `id` must be **lowercase** ASCII alphanumeric plus `-` and `_`, at most
128 characters, and unique within the pipeline. Examples: `build`, `run-tests`,
`deploy_prod`.

## Steps: `script` vs `exec`

Every node runs exactly one of two step kinds.

### `script` — the ergonomic default

A (possibly multi-line) shell script. Runs with **fail-fast** semantics: a
failing line aborts the script with a non-zero exit. Choose the shell:

- `sh` — POSIX `/bin/sh`, always present in the agent image (default).
- `bash` — `/bin/bash`, requires bash in the agent's base image.

```sh
set -eu
echo "building..."
make build
make test
```

### `exec` — direct process, no shell

A command resolved via `PATH` plus a literal argument vector. No shell is
involved, so there is no word-splitting, globbing, or injection surface — the
arguments are passed through verbatim. Reach for this when you want determinism
or are running untrusted input.

| Kind     | Runs via   | Best for                                              |
|----------|-----------|-------------------------------------------------------|
| `script` | a shell   | Ergonomics: pipelines, loops, multiple commands.      |
| `exec`   | direct    | Determinism and safety: one program, explicit argv.   |

## Environment variables

Each node carries its own `env` overlay. A variable's value is either an inline
**literal** or a reference to a project **secret**, resolved and decrypted
control-plane-side at dispatch — the agent never sees the secret store, only the
resolved value. See [Secrets](./secrets.md).

Key rules:

- Must match `^[A-Za-z_][A-Za-z0-9_]*$` (start with a letter or underscore, then
  letters/digits/underscores).
- May **not** start with the reserved prefix `SCYLLA_`.

The agent injects its own context variables under that reserved prefix —
`SCYLLA_WORKSPACE`, `SCYLLA_JOB_ID`, and friends — authoritatively, which is why
your keys can't shadow them.

## The DAG rules

The node set is validated when a pipeline is created or updated. It is rejected
if any of these hold:

- The pipeline has **no nodes**.
- Two nodes share an `id` (**duplicate node ID**).
- A node depends on an `id` that doesn't exist (**invalid dependency**).
- A node lists itself in its own `deps` (**self-dependency**).
- A node lists the same dependency twice (**duplicate dependency**).
- The dependencies form a **cycle**.

Cycle detection uses Kahn's algorithm; ordering is covered in
[Pipeline execution](../architecture/execution.md).

## Worked example

A classic fan-out / fan-in: `setup` runs first; `build`, `test`, and `lint` run
in parallel once setup succeeds; `report` runs after all three finish.

```
        ┌──► build ─┐
setup ──┼──► test  ─┼──► report
        └──► lint  ─┘
```

Node by node:

| Node     | `deps`                   | Step                                 |
|----------|--------------------------|--------------------------------------|
| `setup`  | —                        | `script`: prepare the workspace       |
| `build`  | `setup`                  | `script`: compile                     |
| `test`   | `setup`                  | `script`: run the test suite          |
| `lint`   | `setup`                  | `script`: run the linter              |
| `report` | `build`, `test`, `lint`  | `script`: aggregate the results       |

Because `build`, `test`, and `lint` each depend only on `setup`, the agent runs
them concurrently, then waits for all three before starting `report`.

## Workspaces

All nodes of a single job share one workspace directory (`<root>/<job_id>` on the
agent). Artifacts a node writes are therefore visible to its downstream nodes —
`build` can leave a binary that `report` reads. The workspace is created when the
job starts and removed when it ends (unless the agent is told to keep it for
debugging). Use `working_dir` to run a node in a subdirectory of that workspace.

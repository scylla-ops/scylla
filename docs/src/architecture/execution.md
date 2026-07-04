# Pipeline execution

A `RunPipeline` from the control plane down to running processes on an agent,
and the status/logs that come back.

## Dispatch

Running a pipeline mints a **job**. The control plane picks an agent that is
**connected** (open worker stream) and **authorized** (its App holds an agent
grant covering the pipeline's scope), then sends a `JobDispatch` down that
stream. Secret-sourced env vars are resolved and decrypted control-plane-side
*before* dispatch — the agent receives only literals. Execution is
**sequential per agent**: one job finishes before the next is accepted.

## Topological execution

The agent walks the DAG in dependency order using **Kahn's algorithm**, driven
off node in-degrees:

1. Nodes with no unmet dependency form the ready set (a `BTreeSet`, so
   ordering is **deterministic**).
2. All ready nodes in a batch run **in parallel**, each in its own task.
3. As a node completes, its dependents' in-degrees drop; any that reach zero
   join the next ready batch.

The same algorithm validated the DAG at creation time (a leftover in-degree
means a cycle). See
[Writing pipelines](../using/pipelines.md#the-dag-rules).

## Node processes

Each node runs as a child process inside the job's shared workspace
(`<workspace-root>/<job-id>`, created before the run, removed after unless
`--keep-workspace`; a per-node `working_dir` selects a subdirectory):

- **`script`** steps are materialised to `<workspace>/.scylla/<node>.sh` and
  run from a file (not `-c`) so line numbers are correct and there's no
  `ARG_MAX` limit — `sh -e` or `bash --noprofile --norc -o pipefail -e`
  (fail-fast).
- **`exec`** steps run the command directly via `PATH` with a literal argv —
  no shell.

The environment is **cleared** first (the agent's own token/secrets never
leak into a job), then a minimal allowlist is restored (`PATH`, `HOME`,
`LANG`, `LC_ALL`), `CI=true` / `TERM=dumb` are set, the node's own env vars
are overlaid, and the reserved `SCYLLA_WORKSPACE` / `SCYLLA_JOB_ID` /
`SCYLLA_NODE_ID` are injected last (authoritative). Each node runs in its own
**process group** so cancellation can signal the whole subtree, and its
working directory is canonicalised and checked to be inside the workspace
(defeating symlink escapes).

## Status & log fan-out

Over the same up-stream the agent emits, per node, `NodeStarted` →
(`stdout`/`stderr` log lines) → `NodeCompleted` / `NodeFailed`, bracketed by
`JobStarted` … `JobFailed`/`JobCompleted`. Secret-sourced values are
**redacted** (`***`) from every log line before it leaves the agent. The
control plane fans those logs out to any UI tailing them **and** persists
them — all in-process, no broker or separate recorder.

## Failure handling

On the **first** node failure the executor:

1. cancels in-flight sibling tasks via a cancellation token (SIGTERM to the
   process group, then SIGKILL to stragglers);
2. emits `NodeSkipped` for every node not yet in a terminal state;
3. emits a single terminal `JobFailed`.

Even a failure *before* any node runs (e.g. an unwritable workspace root) is
bracketed `JobStarted` → `JobFailed` carrying the cause, so a job assigned to
an agent can never silently strand in `Pending`. If the agent disconnects
mid-run, the job becomes **`Orphaned`**
(see [Job status](../using/concepts.md#jobs)).

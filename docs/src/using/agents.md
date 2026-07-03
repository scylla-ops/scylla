# Running an agent

Agents are the workers that execute pipeline nodes. An **agent** is an
[App](./concepts.md#apps--agents) running the `scylla-agent` binary, connected to
the control plane over a persistent stream. No agent, no execution — a pipeline
only dispatches to a connected worker.

## 1. Create an App

Agents authenticate as an App, so create one first (in the UI, under your
organization):

1. Create an App. Scylla mints an **app id** and a **one-time secret**.
2. Copy the secret now — it is shown once and stored only as a hash. If you lose
   it, rotate the secret rather than trying to recover it.

The App is granted the `organization-agent` role, which carries exactly the
capabilities a worker needs: read pipelines, execute jobs, and report status and
logs — nothing else.

## 2. Launch the agent

Run the binary with the App's credentials. It connects to the control plane,
exchanges the secret for a short-lived bearer token, and opens its worker stream.

With Docker:

```sh
docker run --rm \
  -e SCYLLA_CONTROL_PLANE_URL=http://<control-plane-host>:50051 \
  -e SCYLLA_APP_ID=<app-id> \
  -e SCYLLA_APP_SECRET=<app-secret> \
  -v scylla-workspaces:/var/lib/scylla/workspaces \
  <docker-user>/scylla-agent:latest
```

Or from a checkout:

```sh
cargo run -p scylla-agent -- \
  --control-plane-url http://127.0.0.1:50051 \
  --app-id <app-id> \
  --app-secret <app-secret>
```

The UI shows ready-to-paste commands with the id and secret already filled in.
Once the stream is open, the App shows as **connected** and is eligible for work.

## 3. Options

Every flag has an environment-variable equivalent (shown in the
[configuration reference](../reference/configuration.md#agent-cli-flags)). The
ones you're most likely to touch:

| Flag | Default | Purpose |
|------|---------|---------|
| `--control-plane-url` | `http://127.0.0.1:50051` | Where to connect. |
| `--app-id` / `--app-secret` | — | App credentials (required). |
| `--workspace-root` | `/var/lib/scylla/workspaces` | Parent dir for per-job workspaces. |
| `--keep-workspace` | `false` | Keep a job's workspace on disk after it ends (debugging). |
| `--max-reconnect-attempts` | `10` | Consecutive failed reconnects before exit (`0` = forever). |
| `--reconnect-backoff-secs` | `3` | Base delay between reconnect attempts. |

## How presence & dispatch work

**Presence is the open stream** — there are no heartbeats. When the stream is up,
the agent is connected; when it drops, the agent is offline and any job it was
running becomes `Orphaned`.

If the stream drops, the agent reconnects with exponential backoff (the base
delay doubles per consecutive failure, capped at 60s). It gives up after
`--max-reconnect-attempts` consecutive short-lived connections. Two failures are
treated as **terminal** and stop the agent immediately rather than retrying — a
revoked secret, a disabled App, or a deleted agent. Retrying can't fix bad
credentials.

Job execution is **sequential** in this version: an agent finishes one job before
accepting the next.

## What the agent does with a job

When a job is dispatched, the agent:

1. Creates a shared workspace for the job at `<workspace-root>/<job-id>`.
2. Walks the pipeline DAG in dependency order, running independent nodes in
   parallel and streaming each node's stdout/stderr back as logs.
3. On the first node failure, cancels its running siblings and marks the
   remaining nodes skipped, then reports the job failed.
4. Removes the workspace when the job ends (unless `--keep-workspace` is set).

Each node runs with a **cleared environment** — the agent's own secrets and
bearer token never leak into a job. Scylla restores a minimal allowlist
(`PATH`, `HOME`, `LANG`, `LC_ALL`), sets `CI=true` and `TERM=dumb`, overlays the
node's own env vars, and injects the reserved context vars `SCYLLA_WORKSPACE`,
`SCYLLA_JOB_ID`, and `SCYLLA_NODE_ID`. Values sourced from secrets are redacted
(`***`) from log output. The mechanics are covered in
[Pipeline execution](../architecture/execution.md).

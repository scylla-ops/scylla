# Running an agent

An **agent** is an [App](./concepts.md#apps--agents) running the
`scylla-agent` binary, connected to the control plane over a persistent
stream. No agent, no execution — a pipeline only dispatches to a connected
worker.

## 1. Create an App

Agents authenticate as an App; create one in the UI, under your organization:

1. Create an App. Scylla mints an **app id** and a **one-time secret**.
2. Copy the secret now — it is shown once and stored only as a hash. If you
   lose it, rotate it; it cannot be recovered.

The App is granted the `organization-agent` role, which carries exactly what a
worker needs — read pipelines, execute jobs, report status and logs — and
nothing else.

## 2. Launch the agent

The agent exchanges the App secret for a short-lived bearer token and opens
its worker stream. The UI shows ready-to-paste commands with the id and secret
filled in.

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

Once the stream is open, the App shows as **connected** and is eligible for
work. All flags (workspace root, `--keep-workspace`, reconnect tuning, …) and
their environment-variable equivalents are in the
[configuration reference](../reference/configuration.md#agent-cli-flags).

## Presence & dispatch

**Presence is the open stream** — there are no heartbeats. When the stream
drops, the agent is offline and any job it was running becomes `Orphaned`.

After a drop the agent reconnects with exponential backoff
(`--reconnect-backoff-secs` base, doubling per consecutive failure, capped at
60s) and gives up after `--max-reconnect-attempts` consecutive failures
(`0` = forever). Credential-class failures — a revoked secret, a disabled App,
a deleted agent — are **terminal** and stop the agent immediately: retrying
can't fix bad credentials.

Job execution is **sequential** in this version: an agent finishes one job
before accepting the next.

## What the agent does with a job

1. Creates the job's shared workspace at `<workspace-root>/<job-id>`.
2. Walks the pipeline DAG in dependency order, running independent nodes in
   parallel and streaming each node's stdout/stderr back as logs.
3. On the first node failure, cancels running siblings, marks the remaining
   nodes skipped, and reports the job failed.
4. Removes the workspace when the job ends (unless `--keep-workspace`).

Each node runs with a cleared, minimal environment — the agent's own secrets
and token never leak into a job — and secret-sourced values are redacted from
logs. The mechanics are in
[Pipeline execution](../architecture/execution.md).

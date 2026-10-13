# Architecture

Scylla has three parts: a control plane, agents and a PostgreSQL database.

```
 browser ──gRPC-Web──┐
 gRPC clients ─gRPC──┤
 webhook senders ────┼──> scylla-ce :8080 ───> PostgreSQL
 agents ───stream────┘
```

## Control plane: `scylla-ce`

One binary, the Community Edition. It serves everything on one port, `8080`
by default:

- the web UI, compiled into the binary;
- the gRPC API, and its gRPC-Web translation for the browser;
- gRPC reflection, so tools such as `grpcurl` can list the services;
- the agent stream (`AgentService.Open`);
- the webhook ingress, `POST /webhooks/{trigger_id}`;
- `GET /healthz`, which answers `ok`.

The UI is served from the same origin as the API, so it needs no CORS setup
and no URL of its own. TLS is terminated by a reverse proxy in front, or by
the binary itself with `[server.tls]` (see [Configuration](./configuration.md)).

Job dispatch, the cron scheduler and the live log fan-out run inside this
process. There is no message broker.

## Agents: `scylla-agent`

A worker installed on each machine that runs jobs. An agent is an *App*: a
machine account of an organization, with an id and a secret. At start, the
agent exchanges its id and secret for an app token. Then it opens one
long-lived bidirectional gRPC stream to the control plane, on the same port.

On that stream the control plane sends jobs and cancel orders. The agent sends
job and node status, and log lines. The open stream is the presence of the
agent; there is no heartbeat message. The agent only makes outbound
connections, so it can run behind NAT or a firewall.

The control plane gives a job to a connected agent that holds `executeJob` on
the pipeline of the job and has no other unfinished job. See
[Running an agent](./running-an-agent.md).

## Database

PostgreSQL holds all state: users, organizations, projects, pipelines, jobs
and their logs, secrets (encrypted), triggers and grants. The control plane
applies the schema migrations at start when `[database].run_migrations` is
`true`.

## Data flow of one run

1. A user or an App calls `RunPipeline`, or a trigger fires. The control plane
   resolves the secrets and the trigger inputs of the pipeline, and stores a
   pending job.
2. The dispatcher places the job on a free agent and sends it on the agent's
   stream.
3. The agent creates a workspace for the job and runs the nodes in DAG order.
   It reports each state change and each log line on the stream.
4. The control plane stores the status and the logs, and pushes the lines to
   the clients that follow the job.

## See also

- The [README](https://github.com/scylla-ops/scylla#readme) for the workspace
  layout and the edition mechanism.
- The [Glossary](./glossary.md) for the terms used in this book.

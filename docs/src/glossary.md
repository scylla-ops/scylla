# Glossary

The terms of this book. The [full glossary](https://github.com/scylla-ops/scylla/blob/main/GLOSSARY.md)
in the repository also covers the terms of the code.

**Agent.** An [App](#app) that runs the `scylla-agent` binary and holds an open
stream to the control plane. It runs jobs on its machine, one at a time. See
[Running an agent](./running-an-agent.md).

<a id="app"></a>**App.** A machine account owned by one organization. It
authenticates with a secret, which it exchanges for an app token valid 30
days. It acts under its grants, like a user.

**Connected.** An agent is connected while its stream to the control plane is
open. There is no heartbeat message: the open stream is the presence.

**Control plane.** The `scylla-ce` process: web UI, gRPC API, webhook ingress,
job dispatch and cron scheduler, on one port. See
[Architecture](./architecture.md).

**DAG.** Directed acyclic graph. The nodes of a pipeline and their `deps` form
one. A cycle is rejected when you create or update the pipeline.

**Delivery.** One POST of a webhook sender to a trigger URL. A delivery id in
`X-Scylla-Delivery` or `X-GitHub-Delivery` makes repeats fire once.

**Grant.** "This principal holds this role on this scope." The only source of
access. See the [Access model](./access-model.md).

**Input.** An environment variable that a trigger adds to every node of the
run: a literal, or a JSON pointer into the webhook payload.

**Job.** One run of a pipeline. It keeps the nodes of the pipeline as they were
when it was created. A job is pending, running, or terminal with an outcome:
`COMPLETED`, `FAILED`, `CANCELLED` or `ORPHANED`.

**Master key.** The 64-hex-character key that encrypts project secrets and
webhook signing secrets. `[secrets].master_key` or `SCYLLA_MASTER_KEY`.

**Node.** One step of a pipeline: an `exec` command or a `script`, with its
id, its deps, its working directory and its environment. In a job, a node ends
as `COMPLETED`, `FAILED`, `CANCELLED` or `SKIPPED`.

**Organization.** The top-level tenant. It owns projects and Apps.

**Orphaned.** The outcome of a job whose agent stayed disconnected for 60
seconds after its last contact, or reconnected without the job.

**Pending.** A job that no agent has started yet. It waits for a connected
agent that may run it and is free.

**Pipeline.** A DAG of nodes in a project. Running it creates a job. See
[Pipelines](./pipelines.md).

**Project.** A unit of work inside an organization. It holds pipelines, jobs,
triggers and secrets.

**Role.** A named set of permissions, valid at one scope kind: System,
Organization or Project.

**Secret.** A value stored encrypted in a project, for example a token. A node
refers to it by name with `secret_ref`. No API returns the value. The agent
masks it in the logs.

**Signing secret.** The 64-hex-character key of a webhook trigger. The sender
signs each body with it (HMAC-SHA256).

**Skipped.** The outcome of a node that did not run, or that the agent
stopped, because another node of the job failed.

**Trigger.** A cron schedule or a webhook URL that starts runs of one
pipeline. See [Triggers](./triggers.md).

**Trigger runner.** The App of an organization that fired runs execute as. Its
name is `trigger-runner`; it may only run pipelines.

**Workspace.** The directory `<workspace root>/<job_id>` on the agent. All
nodes of a job share it; the agent removes it when the job ends.

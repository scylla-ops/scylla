# Getting started

From an empty machine to a pipeline running its first job. Everything runs in
Docker — no Rust, Node.js, or database needed on the host.

## Prerequisites

- **Docker** (>= 24) with **Docker Compose** v2
- **[just](https://github.com/casey/just)** (optional — it wraps the
  `docker compose` commands shown below)

## Start the stack

```sh
git clone https://github.com/scylla-ops/scylla.git
cd scylla
just up
# or, without just:
docker compose pull
docker compose up -d
```

This pulls the prebuilt multi-arch images (amd64 and arm64) and starts:

| Service                | Port    | Role                               |
|------------------------|---------|-------------------------------------|
| `scylla-frontend`      | `8080`  | Web UI                              |
| `scylla-control-plane` | `50051` | gRPC API + in-process job dispatch  |
| `scylla-control-plane` | `8088`  | Inbound webhook ingress             |
| `postgres`             | `5432`  | Datastore                           |

> **Coming from an earlier beta?** Run `just clean` first — it removes old
> containers, volumes, and locally-built images, which are not compatible
> across betas.

Agents are **not** part of this stack — you add them from the UI afterwards.
See [Running an agent](./agents.md).

## Sign in

The first boot creates the bootstrap `admin` user. Open
**<http://localhost:8080/>** and sign in with `admin` / `admin123`.

> Development defaults — change them before exposing Scylla beyond localhost.
> See [Security](../operating/security.md).

## Run your first pipeline

Scylla organizes work as **Organization → Project → Pipeline**; running a
pipeline produces a **job**.

1. After login you land in your active org (the URL carries its slug).
2. **Create a project**, then **a pipeline** in it. A pipeline is a set of
   *nodes*; each runs a command and may depend on other nodes. The smallest
   useful pipeline is one node running `echo "hello from scylla"`.
3. **Add an agent** — see [Running an agent](./agents.md). A pipeline only
   dispatches to a **connected** agent.
4. **Run the pipeline.** Open the resulting job to watch per-node status and
   streamed logs live.

## Next steps

- [Core concepts](./concepts.md) — the vocabulary behind what you just did.
- [Writing pipelines](./pipelines.md) — multi-step DAGs, `exec` vs `script`,
  environment variables, and secrets.

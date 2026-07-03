# Getting started

This chapter takes you from an empty machine to a pipeline running its first job.
Everything runs in Docker — you do **not** need Rust, Node.js, or a database
installed on the host.

## Prerequisites

- **Docker** (>= 24) with **Docker Compose** v2
- **[just](https://github.com/casey/just)** (optional — it just wraps the
  `docker compose` commands shown below)

Check your versions:

```sh
docker --version
docker compose version
```

## Start the stack

Clone the repository and bring the stack up. `just up` pulls the prebuilt,
multi-arch images (amd64 and arm64) and starts three services — the control
plane, PostgreSQL, and the web UI:

```sh
git clone https://github.com/scylla-ops/scylla.git
cd scylla
just up
# or, without just:
docker compose pull
docker compose up -d
```

The stack that comes up:

| Service                | Port    | Role                                            |
|------------------------|---------|-------------------------------------------------|
| `scylla-frontend`      | `8080`  | Web UI                                           |
| `scylla-control-plane` | `50051` | gRPC API + in-process job dispatch              |
| `scylla-control-plane` | `8088`  | Inbound webhook ingress                          |
| `postgres`             | `5432`  | Datastore                                        |

> **Coming from an earlier beta?** Wipe the previous stack first with
> `just clean` — it removes old containers, volumes, and locally-built images,
> which are not compatible across betas.

Agents are **not** part of this stack — you add them from the UI afterwards. See
[Running an agent](./agents.md).

## Sign in

The first boot automatically creates the bootstrap `admin` user. Open
**<http://localhost:8080/>** and sign in:

- **username:** `admin`
- **password:** `admin123`

> These are development defaults. Change them before exposing Scylla beyond
> localhost — see [Security](../operating/security.md).

## Run your first pipeline

Scylla organizes work as **Organization → Project → Pipeline**. A pipeline is a
graph of steps; running it produces a **job**. The short path:

1. **Pick an organization.** After login you land inside your active org (the
   URL carries its slug).
2. **Create a project** to hold your pipelines.
3. **Create a pipeline** in that project. A pipeline is a set of *nodes*; each
   node runs a command, and nodes can depend on other nodes. The smallest useful
   pipeline is a single node running a shell script:

   ```sh
   echo "hello from scylla"
   ```

4. **Add an agent.** A pipeline needs a worker to execute on. Follow
   [Running an agent](./agents.md) to register one — it takes a minute. A
   pipeline only dispatches to a **connected** agent.
5. **Run the pipeline.** This creates a job. Open the job to watch per-node
   status and streamed logs live.

That's the whole loop: define a pipeline once, run it many times, watch each job.

## Next steps

- [Core concepts](./concepts.md) — the vocabulary behind what you just did.
- [Writing pipelines](./pipelines.md) — multi-step DAGs, `exec` vs `script`,
  environment variables, and secrets.
- [Running an agent](./agents.md) — register and launch a worker.

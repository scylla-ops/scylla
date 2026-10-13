# Getting started

This page takes you from nothing to a pipeline that runs on your own agent:

1. Start the control plane.
2. Sign in as the administrator.
3. Create an organization.
4. Create and start an agent.
5. Create a project and a pipeline.
6. Run the pipeline and read its logs.

To start runs from GitHub, continue with [Triggers](./triggers.md).

## 1. Start the control plane

You need Docker (24 or later) with Docker Compose v2. The images are published
for `linux/amd64` and `linux/arm64`.

```sh
git clone https://github.com/scylla-ops/scylla.git
cd scylla
docker compose pull
docker compose up -d
```

With [just](https://github.com/casey/just) installed, `just up` does the same.

The stack has two services: `postgres` (PostgreSQL 18) and `scylla-ce` (the
control plane). The control plane serves the web UI, the gRPC API and the
webhooks on port `8080`. It reads `binaries/scylla-ce/config/docker.toml`,
which the compose file mounts into the container.

Check that it is up:

```sh
curl http://localhost:8080/healthz
# ok
```

### Before you expose it

`docker.toml` is a development configuration. Before other people or the
internet can reach the instance:

- **Set a master key.** Project secrets and webhook signing secrets are
  encrypted with it. The compose file passes `SCYLLA_MASTER_KEY` to the
  container. Left empty, the server uses the public development key from
  `docker.toml`, and your secrets are not confidential. Generate a key of 64
  hex characters and put it in a `.env` file next to `docker-compose.yaml`:

  ```sh
  echo "SCYLLA_MASTER_KEY=$(openssl rand -hex 32)" >> .env
  ```

  Set it before you create secrets. The server cannot decrypt the secrets
  that it encrypted with another key.

- **Change the bootstrap password and email** in the `[bootstrap]` section of
  `docker.toml` before the first start. The server creates the account at the
  first start. If the account exists, a new password in the file does not
  change it.

- **Set `[server].public_url` and `[webhook].public_base_url`** to the public
  address of the instance, so that password reset links and webhook URLs
  point to it.

See [Configuration](./configuration.md) for every key.

### Without Docker

`scylla-ce` is one binary. Build it from a checkout (this needs the Rust
toolchain, Node.js and pnpm):

```sh
git submodule update --init --recursive
just ui-build
cargo build --release -p scylla-ce
./target/release/scylla-ce --config my-config.toml
```

It needs a PostgreSQL database and applies its migrations at start when
`[database].run_migrations` is `true`. `binaries/scylla-ce/config/prod.toml` is
a starting point for `my-config.toml`. It has no `[bootstrap]` and no
`[secrets]` section: add them, or no administrator is created and secrets are
disabled. `[bootstrap]` needs a `username`, a `password` and an `email`.

## 2. Sign in

Open <http://localhost:8080/> and sign in with the bootstrap account:

- username: `admin`, or the email `admin@example.com`
- password: `admin123` (or the password you set)

This account holds the System Admin role: it may do everything, everywhere.

## 3. Create an organization

An organization holds projects, agents and the roles of its people. Create one
with **Create Organization**. As its creator, you get the Organization Admin
role on it.

## 4. Create and start an agent

An agent is a worker process, `scylla-agent`, that runs jobs on a machine of
yours. In Scylla it is an *App*: a machine account of the organization with
an id and a secret.

1. Open **Agents** and click **New Agent**.
2. Give it a name and click **Create & reveal secret**.
3. Copy the secret. It is shown only once; the server stores only its hash.

The new agent holds the Organization Agent role on the organization. It may
run the jobs of every project of the organization.

The UI then shows two ready-to-paste commands, with the app id and the secret
filled in. With Docker:

```sh
docker run godlyjaaaaj/scylla-agent:latest \
  --control-plane-url http://host.docker.internal:8080 \
  --app-id <APP_ID> \
  --app-secret <APP_SECRET>
```

From a checkout, with the Rust toolchain:

```sh
cargo run --release --bin scylla-agent -- \
  --control-plane-url http://localhost:8080 \
  --app-id <APP_ID> \
  --app-secret <APP_SECRET> \
  --workspace-root "$HOME/.scylla/workspaces"
```

`--control-plane-url` is the address at which the agent reaches the control
plane. In a container, `localhost` is the container itself. On Linux,
`host.docker.internal` needs `--add-host=host.docker.internal:host-gateway` on
the `docker run` command line.

The agent runs the steps of your pipelines on its own machine, with the tools
installed there. The `scylla-agent` image is a Debian bookworm slim image with
`curl` and CA certificates added. It does not have `git`, `make` or a
compiler. For a real build, run the agent on a host that has your tools, or
build your own image on top of it.

The agent page shows **connected** once the agent holds its stream open. See
[Running an agent](./running-an-agent.md) for the flags, the workspace and the
reconnection rules.

## 5. Create a project and a pipeline

1. Open **Projects** and click **New project**. As its creator, you get the
   Project Admin role on it.
2. Open the project and click **New pipeline**.
3. The editor starts with a two-node template. Click **Create**.

The editor has a graphical mode and a text mode. The text mode takes a JSON
document. [Pipelines](./pipelines.md) describes every field, and has a larger
example.

## 6. Run it and read the logs

Click the run button (the triangle) of the pipeline. The server creates a job:

- If a connected agent that may run the job is free, the job goes to it at
  once.
- Otherwise the job stays pending until such an agent is free. An agent runs
  one job at a time.

Open the job from the history of the pipeline. The job page shows each node,
its state and its log lines, live while the job runs. A job ends as
`COMPLETED`, `FAILED`, `CANCELLED` or `ORPHANED`; see
[Pipelines](./pipelines.md#job-and-node-outcomes).

If the job stays pending, check that the agent shows as connected and that it
belongs to the organization of the project.

## Next steps

- Store tokens and passwords as project secrets, and refer to them by name
  with `secret_ref`: [Pipelines](./pipelines.md#environment).
- Start runs on a schedule or from a GitHub push:
  [Triggers](./triggers.md).
- Give other people access: see [Add people](#add-people) below.

## Add people

The Community Edition has no sign-up and no invitations, and it sends no mail.
A system administrator adds people:

1. Create the account with **New user** (`UserService.CreateUser`): a
   username, an email and an initial password.
2. In the organization or the project, use **Add a member** to give the user a
   role (`GrantService.CreateGrant`). The dialog picks the user in the user
   directory, which only a system administrator can read.

To reset a password, the user clicks **Forgot password?** on the sign-in page,
or an administrator sends a reset link to the user
(`UserService.SendPasswordReset`). The server writes the link in its log, at
INFO, with the message `password reset link`, the user id and the username.
Find it with `docker compose logs scylla-ce` and give it to the user. A link
works once and expires after one hour. Using it signs out every session of
the account.

Each user sees their own sessions, with the device and the IP address, and
can sign one or all of them out.

See the [Access model](./access-model.md#adding-a-member) for the rules.

## Who needs which role

Each task needs a permission on an organization, a project or a pipeline. The
table gives the smallest builtin role that grants it. A role on an
organization covers all its projects; System Admin covers everything. See the
[Access model](./access-model.md) for the full rules.

| Task                                   | Permission                       | Smallest builtin role         |
| -------------------------------------- | -------------------------------- | ----------------------------- |
| Create an organization                 | `createOrganization`             | Organization Creator (System) |
| Create a project                       | `createProject` on the organization | Organization Admin         |
| Create an agent                        | `createAgent` on the organization | Organization Admin           |
| Create, edit, delete a pipeline        | `createPipeline`, `updatePipeline`, `deletePipeline` | Project Developer |
| Run a pipeline                         | `runPipeline` on the pipeline    | Project Developer             |
| Cancel a job                           | `updateJob` on the job           | Project Developer             |
| Read jobs and logs                     | `readJob`, `readJobLogs`         | Project Viewer                |
| Create a secret                        | `createSecret` on the project    | Project Admin                 |
| Delete a secret                        | `deleteSecret` on the secret     | Project Admin                 |
| List secret names                      | `listSecrets` on the project     | Project Developer             |
| Create or change a trigger             | `manageTriggers` and `runPipeline` on the pipeline | Project Developer |
| Enable, disable or delete a trigger    | `manageTriggers` on the trigger  | Project Developer             |
| Fire a trigger now                     | `runPipeline` on the trigger     | Project Developer             |
| Run jobs (an agent)                    | `executeJob` on the pipeline     | Project Agent or Organization Agent |

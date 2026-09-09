# Scylla

Distributed CI/CD platform.

## Architecture

| Service                | Ports          | Description                                                       |
|------------------------|----------------|-------------------------------------------------------------------|
| `scylla-control-plane` | `8080`         | One binary: web UI, gRPC API, gRPC-Web, webhook ingress, and in-process job dispatch |
| `scylla-agent`         | —              | Worker installed per machine; registered as an "App" in the UI, run out-of-band |
| `postgres`             | `5432`         | Primary datastore (PostgreSQL 18)                                 |

Two binaries ship: `scylla-control-plane` (central brain) and `scylla-agent` (remote workers). Agents connect to the control plane over a persistent worker stream, there is no message broker.

Everything the browser and the agents talk to lives on **one port**. The control plane serves the compiled-in web UI, terminates gRPC-Web for the browser, speaks native gRPC to the agents, and accepts inbound webhooks at `/webhooks/{trigger_id}` — all on `8080`. Because the UI is served from the same origin as the API, the bundle uses relative URLs: the published image carries no baked-in hostname and works unchanged in any deployment.

The workspace separates what ships from what is shared. `binaries/` holds the two packages that produce an executable — `scylla-control-plane` (use cases, adapters, the HTTP/gRPC surface, and the binary) and `scylla-agent` (the worker binary). `crates/` holds the two libraries both of them link: `scylla-domain` (the shared kernel — the domain model plus the types the two binaries exchange) and `scylla-proto` (the `.proto` files and their generated bindings, which the frontend also consumes). `scylla-domain` deliberately links no database, no gRPC stack and no crypto, so an agent can depend on it cheaply.

## Prerequisites

You only need Docker. Everything else (Rust, Node.js, pnpm) runs inside containers.

- **Docker** (>= 24) with **Docker Compose** v2
- **[just](https://github.com/casey/just)** (optional, shortcuts for `docker compose`)

Check versions:

```sh
docker --version
docker compose version
```

## Quick start

Prebuilt images are published for both `linux/amd64` and `linux/arm64` — Docker pulls the right one for your host automatically.

One command pulls the prebuilt images and starts the stack (control plane, PostgreSQL). Agents are added afterward from the UI — see below:

> [!WARNING]
> Coming from an earlier beta? Wipe the previous stack first with `just clean` — it removes the old containers, volumes, and locally-built images, which are not compatible across betas.

```sh
git clone https://github.com/scylla-ops/scylla.git
cd scylla
just up
# or, without just:
docker compose pull
docker compose up -d
```

First boot creates the `admin` user automatically.

Open **http://localhost:8080/** and sign in:

- username: `admin`
- password: `admin123`

## Developing

The stack above is enough to run Scylla. To work on it:

```sh
just db-up                                        # Postgres alone
cargo run -p scylla-control-plane -- \
    --config binaries/scylla-control-plane/config/local.toml --no-ui
cd apps/frontend && pnpm install && pnpm dev       # http://localhost:5173
```

`--no-ui` is what makes this work: the web UI is compiled into the release
binary, and in dev there is nothing to compile in — Vite owns it on `:5173`.
The flag drops the UI routes entirely so the binary serves only the API.
(`config/local.toml` already sets `[ui].enabled = false`, so the flag is
belt-and-braces.) `apps/frontend/.env` points the dev bundle at `:8080`;
`local.toml` allows that origin through CORS.

To run the real thing natively, build the UI first — a release `cargo build`
embeds whatever is in `apps/frontend/dist`:

```sh
just ui-build
cargo build --release -p scylla-control-plane
```

## TLS

The control plane speaks plain HTTP by default, on the assumption that a reverse
proxy or ingress terminates TLS in front of it. To terminate it in the binary
instead, point `[server.tls]` at a PEM chain and key:

```toml
[server]
address = "0.0.0.0:8443"

[server.tls]
cert = "/etc/scylla/tls/fullchain.pem"
key  = "/etc/scylla/tls/privkey.pem"
```

Both HTTP/2 and HTTP/1.1 are advertised over ALPN, so browsers, gRPC clients and
plain HTTP/1.1 webhook senders all connect to the same socket.

## Common commands

| `just`        | `docker compose`                  | What it does                                  |
|---------------|-----------------------------------|-----------------------------------------------|
| `just up`     | `docker compose pull && up -d`    | Pull and start (or refresh) the beta stack    |
| `just down`   | `docker compose down`             | Stop the stack                                |
| `just clean`  | `docker compose down -v --rmi local --remove-orphans` | Stop and wipe volumes + local images |
| `just logs [svc]` | `docker compose logs -f [svc]` | Follow logs (all services or one)            |
| `just status` | `docker compose ps`               | Show running containers                       |

Run `just --list` to see every recipe.

## Troubleshooting

**Port already in use.** Another process holds `8080` or `5432`. Stop it or change the host port in `docker-compose.yaml`.

**`scylla-control-plane` fails to connect to PostgreSQL.** Ensure `postgres` is `healthy` via `just status` (or `docker compose ps`). If it's stuck, run `just clean` to reset the volume and try again.

**Frontend shows gRPC errors.** The UI and the API share an origin, so there is no CORS step to get wrong — check that `scylla-control-plane` is `healthy` (`just status`) and read its logs. `curl http://localhost:8080/healthz` should answer `ok`.

**Agent not picking up jobs.** Agents run out-of-band (not in this compose stack). Check the agent's own logs and confirm it can reach the control plane at its `--control-plane-url` with a valid `--app-id` / `--app-secret`. In the UI the app shows as connected once its worker stream is open.

> Still stuck? Open a post in the `help` Discord channel with:
>  - steps to reproduce
>  - `docker compose logs` output for the affected service
>  - `docker compose ps` snapshot

## Optional: local development

The Docker workflow above is self-contained — **Node.js and Rust are not required** to run Scylla. The sections below are only for contributors who want to iterate on the frontend or crates outside Docker.

### Frontend (Vite dev server on `:5173`)

Requires Node.js >= 20 and pnpm >= 9 (`corepack enable`). See [apps/frontend/README.md](apps/frontend/README.md) for setup.

### Building crates locally

Requires the Rust toolchain. Standard Cargo workflow:

```sh
cargo build
cargo test
```

To build the Docker images from source instead of pulling them:

```sh
just local
```

## Further reading

- [Glossary](GLOSSARY.md), every Scylla-specific term, grouped by topic.
- Releasing images: `just release` builds and pushes the multi-arch Docker images to Docker Hub. Run `just --list` for the individual recipes.

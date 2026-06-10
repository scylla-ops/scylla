# Scylla

Distributed CI/CD platform.

## Architecture

| Service                | Ports          | Description                                                       |
|------------------------|----------------|-------------------------------------------------------------------|
| `scylla-frontend`      | `8080`         | Web UI (Caddy-served static build)                                |
| `scylla-control-plane` | `50051`        | Single binary: gRPC API + in-process job dispatch (the agent worker stream) |
| `scylla-agent`         | —              | Worker installed per machine; registered as an "App" in the UI, run out-of-band |
| `postgres`             | `5432`         | Primary datastore (PostgreSQL 18)                                 |

Two binaries ship: `scylla-control-plane` (central brain) and `scylla-agent` (remote workers). Agents connect to the control plane's gRPC API (`50051`) over a persistent worker stream — there is no message broker. `scylla-api` is a library composed inside the control plane.

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

One command pulls the prebuilt images and starts the stack (control plane, PostgreSQL, frontend). Agents are added afterward from the UI — see below:

```sh
git clone https://github.com/scylla-ops/scylla.git
cd scylla
just up
# or, without just:
docker compose pull && docker compose up -d
```

First boot creates the `admin` user automatically.

Open **http://localhost:8080/** and sign in:

- username: `admin`
- password: `admin123`

## Common commands

| `just`        | `docker compose`                  | What it does                                  |
|---------------|-----------------------------------|-----------------------------------------------|
| `just up`     | `docker compose pull && up -d`    | Pull latest images and start the stack        |
| `just pull`   | `docker compose pull`             | Refresh images without (re)starting           |
| `just update` | `docker compose pull && up -d`    | Pull and recreate containers on a running stack |
| `just down`   | `docker compose down`             | Stop the stack                                |
| `just clean`  | `docker compose down -v --rmi local --remove-orphans` | Stop and wipe volumes + local images |
| `just logs [svc]` | `docker compose logs -f [svc]` | Follow logs (all services or one)            |
| `just status` | `docker compose ps`               | Show running containers                       |

Run `just --list` to see every recipe.

## Troubleshooting

**Port already in use.** Another process holds `8080`, `5432`, or `50051`. Stop it or change the host port in `docker-compose.yaml`.

**`scylla-control-plane` fails to connect to PostgreSQL.** Ensure `postgres` is `healthy` via `just status` (or `docker compose ps`). If it's stuck, run `just clean` to reset the volume and try again.

**Frontend shows gRPC errors.** Verify the UI on `http://localhost:8080` can reach the control plane on `http://localhost:50051`. The browser must accept CORS — the default config already allows `http://localhost:8080`.

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

- [Glossary](GLOSSARY.md) — every Scylla-specific term, grouped by topic.
- [Releasing images](docs/release.md) — how the multi-arch Docker images are built and pushed to Docker Hub (manual, via `just release`).

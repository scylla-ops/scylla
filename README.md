# Scylla

Distributed CI/CD platform.

## Architecture

| Service                | Ports          | Description                                                       |
|------------------------|----------------|-------------------------------------------------------------------|
| `scylla-ce`            | `8080`         | One binary: web UI, gRPC API, gRPC-Web, webhook ingress, and in-process job dispatch |
| `scylla-agent`         | —              | Worker installed per machine; registered as an "App" in the UI, run out-of-band |
| `postgres`             | `5432`         | Primary datastore (PostgreSQL 18)                                 |

Two binaries ship: `scylla-ce` (the control plane, Community Edition) and `scylla-agent` (remote workers). Agents connect to the control plane over a persistent worker stream, there is no message broker.

Everything the browser and the agents talk to lives on **one port**. The control plane serves the compiled-in web UI, terminates gRPC-Web for the browser, speaks native gRPC to the agents, and accepts inbound webhooks at `/webhooks/{trigger_id}` — all on `8080`. Because the UI is served from the same origin as the API, the bundle uses relative URLs: the published image carries no baked-in hostname and works unchanged in any deployment.

The workspace is a stack of library crates under `crates/` with the two binaries under `binaries/`: `scylla-ce` (the control plane) and `scylla-agent` (the worker). See [Workspace layout](#workspace-layout) below.

## Workspace layout

```
crates/
  scylla-extension/   the edition boundary: extension traits + the Extensions bundle (no workspace dependency)
  scylla-domain/      the shared kernel: domain model, JobEvent (no I/O, no crypto)
  scylla-proto/       the wire contract: .proto files + generated bindings (also consumed by the frontend)
  scylla-auth/        the access model: RBAC ports and types, the Cedar adapter
  scylla-core/        use cases and their ports, gRPC + HTTP surfaces, server config, in-memory adapters
  scylla-db/          the Postgres adapters, the pool, the embedded migrations
  scylla-server/      the composition root: serve(config, db, extensions) and the shared cli
binaries/
  scylla-ce/          the Community Edition binary: a main.rs and config/*.toml
  scylla-agent/       the worker binary (depends on scylla-domain + scylla-proto only)
apps/frontend/        the web UI, compiled into scylla-ce through scylla-core
migrations/           the SQL schema, embedded by scylla-db
```

Dependencies point one way, bottom to top:

```
scylla-extension
scylla-domain <- scylla-proto <- scylla-core <- scylla-db <- scylla-server <- scylla-ce
scylla-domain <- scylla-auth  <- scylla-core
```

`scylla-core` is generic over the ports declared in `scylla-core` and `scylla-auth`; `scylla-db` implements them; `scylla-server` is the only crate that names the concrete implementations side by side. The binaries are a `main.rs` each: parse the command line, load the configuration, open the pool with `scylla_db::init_db`, build the edition's `Extensions`, call `scylla_server::serve`.

A WebAssembly plugin runtime is planned as `crates/scylla-wasm/`; it does not exist yet.

## Editions and the extension mechanism

This repository is the open source core and ships the Community Edition binary, `scylla-ce`. A separate, private repository builds an Enterprise binary on top of it. The dependency is strictly one way: the private repository depends on this one by a pinned git tag, and nothing here knows it exists.

The seam between the two is `crates/scylla-extension`: a small crate holding only traits and the types that appear in their signatures, with no dependency on any other workspace crate. An edition implements the traits, bundles the implementations in an `Extensions` value, and hands it to `scylla_server::serve` along with the pool it opened, so an implementation can share the database. The core calls through the traits and never knows which edition built it.

One extension point exists today, the quota:

```rust
#[async_trait]
pub trait QuotaPolicy: Send + Sync {
    async fn check(&self, resource: Resource, scope: &str) -> Result<QuotaDecision, QuotaError>;
    async fn usage(&self, resource: Resource, scope: &str) -> Result<Option<QuotaUsage>, QuotaError>;
}

pub enum QuotaDecision {
    Allow,
    Deny { resource: Resource, limit: u64, current: u64, upgrade_hint: Option<String> },
}

#[derive(Clone)]
pub struct Extensions {
    pub quota: Arc<dyn QuotaPolicy>,
}
```

`ProjectUseCases::create` asks the policy before creating a project and turns a `Deny` into `DomainError::QuotaExceeded` (gRPC `RESOURCE_EXHAUSTED`). The Community Edition wires `UnlimitedQuota` (`binaries/scylla-ce/src/main.rs`), which always allows and consults nothing. `scope` is the organization id as a plain string so the contract stays free of the domain model.

Adding an extension point is: a trait and its boundary types in `scylla-extension`, a field on `Extensions`, a default implementation in `scylla-core`, and a line in each edition's `Extensions` literal.

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
> Coming from an earlier beta? Wipe the previous stack first with `just clean` — it removes the old containers, volumes, and locally-built images, which are not compatible across betas. This release also drops the separate `scylla-frontend` service, and `docker compose down` alone would leave it behind as an orphan.

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
cargo run -p scylla-ce -- \
    --config binaries/scylla-ce/config/local.toml --no-ui
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
cargo build --release -p scylla-ce
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

**`scylla-ce` fails to connect to PostgreSQL.** Ensure `postgres` is `healthy` via `just status` (or `docker compose ps`). If it's stuck, run `just clean` to reset the volume and try again.

**Frontend shows gRPC errors.** The UI and the API share an origin, so there is no CORS step to get wrong; check that `scylla-ce` is `healthy` (`just status`) and read its logs. `curl http://localhost:8080/healthz` should answer `ok`.

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
- Releasing images: see [RELEASING.md](RELEASING.md). `just release <version>` builds and pushes the multi-arch images; `just --list` shows the individual recipes.

## License

Apache License 2.0, see [LICENSE](./LICENSE).

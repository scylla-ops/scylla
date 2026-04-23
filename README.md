# Scylla

Distributed CI/CD platform.

## Architecture

| Service          | Port    | Description                                  |
|------------------|---------|----------------------------------------------|
| `scylla-api`     | `50051` | gRPC API (auth, projects, pipelines, jobs)   |
| `scylla-broker`  | `50052` | Broker for job dispatch and agent presence   |
| `scylla-agent`   | —       | Worker that runs pipeline jobs               |
| `scylla-recorder`| —       | Persists broker events into SurrealDB        |
| `surrealdb`      | `8000`  | Primary datastore                            |
| `frontend`       | `5173`  | Vite dev server                              |

## Prerequisites

Install before starting:

- **Docker** (>= 24) with **Docker Compose** v2
- **Node.js** >= 20 and **pnpm** >= 9 (enable via `corepack enable`)
- **[just](https://github.com/casey/just)** (**optional**, shortcuts for docker compose)
- **Rust** toolchain (**optional**, only to build crates outside Docker)

Check versions:

```sh
docker --version
docker compose version
node --version
pnpm --version
```

## 1. Clone

```sh
git clone https://github.com/scylla-ops/scylla.git
cd scylla
```

## 2. Start everything

Pulls prebuilt images and starts all backend services (API, broker, agent, recorder, SurrealDB).

```sh
docker compose up -d
# or
just up
```

Check everything is healthy:

```sh
docker compose ps
# or
just status
```

Tail logs:

```sh
docker compose logs -f scylla-api
# or
just logs scylla-api
```

First boot creates the `admin` user automatically.

**Default credentials:**

- username: `admin`
- password: `admin123`

## 4. Stop everything

```sh
docker compose down
# or
just down
```

To also wipe the SurrealDB volume:

```sh
docker compose down -v
# or
just clean
```

## Troubleshooting

**Port already in use.** Another process holds `5173`, `8000`, `50051`, or `50052`. Stop it or change the host port in `docker-compose.yaml`.

**`scylla-api` fails to connect to SurrealDB.** Ensure `surrealdb` is `healthy` via `docker compose ps`. If it's stuck, run `docker compose down -v` to reset the volume and try again.

**Frontend shows gRPC errors.** Verify `scylla-api` is reachable on `http://localhost:50051`. Browser must accept CORS, default config already allows `http://localhost:5173`.

**Agent not picking up jobs.** Check `docker compose logs -f scylla-agent` and confirm the broker URL resolves. Restart with `docker compose restart scylla-agent`.

Still stuck? Reach us by openning a post in the `help` Discord channel with:

- steps to reproduce
- `docker compose logs` output for the affected service
- `docker compose ps` snapshot

## Further reading

- [Glossary](GLOSSARY.md) — every Scylla-specific term, grouped by topic.

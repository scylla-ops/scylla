# Deployment

Scylla ships as Docker images and runs as a small Compose stack. This chapter
covers a production-shaped deployment; for a first local run see
[Getting started](../using/getting-started.md).

## The stack

`docker-compose.yaml` defines three services:

| Service | Image | Ports | Role |
|---------|-------|-------|------|
| `postgres` | `postgres:18-alpine` | `5432` | Datastore. |
| `scylla-control-plane` | `scylla-control-plane` | `50051`, `8088` | gRPC API + in-process dispatch; webhook ingress. |
| `scylla-frontend` | `scylla-frontend` | `8080` → `80` | Web UI (Caddy-served static build). |

Agents are **not** in this stack — each agent is an App you register from the UI
and run out-of-band (its credentials only exist after the App is created). See
[Running an agent](../using/agents.md).

## Ports

| Port | Service | Purpose |
|------|---------|---------|
| `8080` | frontend | Web UI. |
| `50051` | control plane | gRPC / gRPC-Web API + the agent worker stream. |
| `8088` | control plane | Inbound webhook ingress (`POST /webhooks/{trigger_id}`). |
| `5432` | postgres | Database. |

The control plane exposes **one** gRPC port — user APIs, app-token exchange, and
the agent stream all share `50051`. There is no message broker.

## Images & tags

Images are published multi-arch (`linux/amd64` + `linux/arm64`); Docker pulls the
one matching your host. Pin a version with the `VERSION` variable rather than
riding `latest`:

```sh
VERSION=0.4.0 docker compose pull
VERSION=0.4.0 docker compose up -d
```

Maintainers build and push the full stack with `just release` (see the `release`
recipe group in the `justfile`).

## Bring it up

```sh
just up            # docker compose pull && up -d
# or
docker compose pull
docker compose up -d
```

Point the control plane at a config file with `--config` (the compose file mounts
`crates/scylla-control-plane/config/` and passes `docker.toml`). See
[Configuration](./configuration.md).

## Frontend build-time config

The UI is a static build. The API URL is **baked in at build time** via the
`VITE_API_URL` build arg — it is not read at runtime. If you serve the API from a
different origin, rebuild the frontend image with the right `VITE_API_URL` (the
`release-frontend` recipe passes it through).

## Production checklist

Before exposing Scylla beyond localhost, work through [Security](./security.md):
change the bootstrap credentials, set a real secret master key, and pin CORS to
your domain.

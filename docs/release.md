# Releasing Scylla images

Images are built and pushed **manually, from a dev machine** — there is no CI
image pipeline. The release recipes are plain `docker buildx` multi-arch
builds of the same Dockerfiles used for local dev ([Dockerfile](../Dockerfile)
for the backend services, [apps/frontend/Dockerfile](../apps/frontend/Dockerfile)
for the frontend), built for `linux/amd64` + `linux/arm64` and pushed to
Docker Hub with their multi-arch manifest.

Trade-off, chosen deliberately: the non-native platform builds under
emulation, so a full backend release from a laptop takes a while. In exchange
the flow is one command, uses a single Dockerfile per image, needs no host
toolchain (only Docker), and behaves identically on any machine.

## One-time setup

```sh
just release-setup   # creates the multi-arch buildx builder
docker login         # account owning the $DOCKER_USER repositories
```

If you already have a docker-container builder (check `docker buildx ls`),
point the recipes at it instead: `BUILDER=multi-builder just release`.

## Releasing

```sh
VERSION=0.3.0 just release            # SaaS stack: control-plane SaaS + agent + frontend
VERSION=0.3.0 just release-saas       # control-plane with --features saas, alone
VERSION=0.3.0 just release-frontend   # frontend only
just release-svc scylla-agent         # one service, VERSION defaults to latest
VERSION=0.3.0 just release-backend    # PaaS edition (control-plane + agent) — NOT part
                                      # of `release` for now, push it explicitly
```

`just release` deliberately skips the PaaS control-plane while the focus is
the SaaS beta. The agent and frontend images have no edition split — the same
images serve both.

Deployment consumes the same variables: the SaaS compose overlay points the
control-plane at `:${VERSION:-latest}-saas`, so `just up` (or
`VERSION=0.3.0 just up`) pulls exactly what the matching `just release`
pushed. The base compose alone stays on the PaaS `:latest` — refresh it with
`just release-backend` when needed.

Knobs (env vars, also read from `.env`): `DOCKER_USER` (default
`godlyjaaaaj`), `VERSION` (default `latest`), `VITE_API_URL` (baked into the
frontend assets at build time, default `http://localhost:50051`), `BUILDER`
(buildx builder name, default `scylla-builder`).

### Tags pushed

| Recipe | Image | Tags |
|---|---|---|
| `release-backend` | `$DOCKER_USER/scylla-control-plane`, `…/scylla-agent` | `:$VERSION`, `:latest` |
| `release-saas` | `$DOCKER_USER/scylla-control-plane` | `:$VERSION-saas`, `:latest-saas`, `:saas` |
| `release-frontend` | `$DOCKER_USER/scylla-frontend` | `:$VERSION`, `:latest` |

## Notes

- **Build cache.** The docker-container builder keeps its BuildKit cache
  locally, and the backend Dockerfile's cargo-chef `deps` stage means a
  release rebuild only recompiles the workspace crates unless
  `Cargo.lock`/`Cargo.toml` changed. The first release on a fresh builder is
  the expensive one.
- **sqlx offline.** Backend builds use the committed `.sqlx/` cache
  (`SQLX_OFFLINE=true` in the Dockerfile). If a build fails with "no cached
  data for this query", run `just db-up && just db-prepare` and commit the
  updated `.sqlx/`.
- **Before tagging a real version**, make sure `.sqlx/` is current
  (`just db-prepare-check`) — the old CI guard that verified this on every
  push is gone.

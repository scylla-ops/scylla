# Local development

Running Scylla needs only Docker. This chapter is for **contributors** who want to
iterate on the crates or the frontend outside the container workflow.

## Backend (Rust)

Standard Cargo workflow against the workspace:

```sh
cargo build
cargo test
```

The toolchain is pinned in `rust-toolchain.toml`, and lints (`clippy::all` +
`pedantic`, `unsafe_code = warn`) are configured at the workspace root — treat a
clean `cargo clippy` as part of "done".

To build the Docker images from source instead of pulling them:

```sh
just local          # docker compose build
```

## Database for dev

The compile-time-checked `sqlx` queries need either a live database or the
committed offline cache. For iterating on queries, run Postgres and point
`DATABASE_URL` at it:

```sh
just db-up          # start only Postgres
just db-migrate     # apply migrations
```

If you change any query or migration, regenerate and commit the cache with
`just db-prepare` (CI enforces it via `just db-prepare-check`). See
[Database & migrations](../operating/database.md).

## Frontend (Vite dev server)

Requires **Node.js >= 20** and **pnpm >= 9** (`corepack enable`). The dev server
runs on `:5173` and talks to a running control plane via `VITE_API_URL`. Setup
lives in `apps/frontend/README.md`; the app's own architecture and conventions are
in `apps/frontend/docs/`.

## The justfile

`just --list` shows every recipe, grouped:

| Group | Recipes | For |
|-------|---------|-----|
| `dev` | `up`, `start`, `down`, `logs`, `status`, `clean`, `local` | Run the stack. |
| `db` | `db-up`, `db-migrate`, `db-revert`, `db-prepare`, `db-prepare-check`, `db-reset` | Database + sqlx cache. |
| `release` | `release`, `release-backend`, `release-frontend`, `release-setup` | Multi-arch image build & push. |

## Before you push

- `cargo test` and `cargo clippy` are clean.
- `.sqlx/` is current (`just db-prepare-check`) if you touched SQL.
- Follow the [conventions](./conventions.md) for new domain code.

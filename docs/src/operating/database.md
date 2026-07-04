# Database & migrations

Scylla stores everything in PostgreSQL (18 in the shipped stack). The schema
is versioned SQL applied automatically at boot.

## Schema & migrations

Migrations are plain SQL files under `migrations/`, applied via
`sqlx::migrate!` when the control plane starts — provided
`[database].run_migrations` is `true` (the default in the shipped configs). To
run them out-of-band instead (e.g. in a controlled release), set
`run_migrations = false` and apply them yourself with `sqlx`.

## The offline query cache

Scylla uses `sqlx`'s compile-time-checked queries. So the code can compile
**without a live database** (CI, Docker builds), the query metadata is cached
in the committed `.sqlx/` directory. If you change any SQL query or migration,
regenerate and commit it:

```sh
just db-prepare          # cargo sqlx prepare --workspace ...
```

CI verifies it with `just db-prepare-check` — a stale cache fails the build.

## Local database workflow

For host-native development you only need Postgres running; the `db` recipe
group wraps the common tasks:

| Recipe | Does |
|--------|------|
| `just db-up` | Start only the Postgres dev container. |
| `just db-migrate` | Apply pending migrations against `$DATABASE_URL`. |
| `just db-revert` | Revert the most recent migration. |
| `just db-prepare` | Regenerate the `.sqlx/` offline cache. |
| `just db-reset` | Drop & recreate the dev volume (**destructive**). |

`DATABASE_URL` defaults to `postgres://scylla:scylla@localhost:5432/scylla`.

## Connection pool

`[database]` tunes the pool: `max_connections`, `min_connections`,
`acquire_timeout`. The shipped `prod.toml` runs a larger pool (32/4) than
local dev (8/1). See the
[reference](../reference/configuration.md#database).

## Upgrades

The compose stack mounts the Postgres **parent** data directory
(`/var/lib/postgresql`), not the version-specific subdirectory. PG 18+ stores
data under `/var/lib/postgresql/<major>/`, so mounting the parent keeps a
future `pg_upgrade --link` to PG 19+ viable without moving the volume.

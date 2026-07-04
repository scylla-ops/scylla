# Configuration

The control plane is configured by a single TOML file, selected with
`--config`, plus a few environment overrides. Every key, type, and default is
tabulated in the [Configuration reference](../reference/configuration.md).

## Config files

Per-environment files live under `crates/scylla-control-plane/config/`:

| File | For |
|------|-----|
| `local.toml` | Host-native development (binds `127.0.0.1`). |
| `docker.toml` | The Compose stack (mounted into the container). |
| `prod.toml` | Production shape (binds `0.0.0.0`, tighter CORS). |

```sh
scylla-control-plane --config /app/config/docker.toml
```

The file has a handful of tables. `[grpc]` and `[database]` are required;
`[cors]`, `[bootstrap]`, and `[metering]` are defaulted; `[secrets]`,
`[webhook]`, `[mail]`, and `[oauth]` are **optional — leaving one out disables
the feature it configures** (no secret store, no webhook server, a no-op
mailer, no OAuth login). The sample files only fill the always-on sections; to
enable secrets, webhooks, email, or OAuth in a real deployment, add the
corresponding table — the [reference](../reference/configuration.md) has the
exact keys.

## Environment overrides

Secrets don't belong in an image, so the compose stack overrides sensitive
values from the environment:

- **`DATABASE_URL`** — overrides `[database].url`. The compose file builds it
  from `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` so the database
  URL never has to be committed.
- **`RUST_LOG`** — log verbosity (e.g. `info`, `debug`, `scylla_core=debug`).

Inject the secret master key and any OAuth/SMTP credentials the same way at
deploy time rather than committing them.

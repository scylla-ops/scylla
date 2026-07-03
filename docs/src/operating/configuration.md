# Configuration

The control plane is configured by a single TOML file, selected with `--config`,
plus a few environment overrides. This chapter explains the shape; every key,
type, and default is tabulated in the
[Configuration reference](../reference/configuration.md).

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

## Sections

The file has a handful of tables. Some are **optional** — leaving one out
disables the feature it configures:

| Section | Required | Configures |
|---------|----------|------------|
| `[grpc]` | yes | The gRPC listen address. |
| `[database]` | yes | PostgreSQL connection + pool + migrations. |
| `[cors]` | defaulted | Allowed origins/methods/headers for the browser. |
| `[bootstrap]` | defaulted | The first-boot admin account. |
| `[metering]` | defaulted | Per-org quotas (e.g. max projects). |
| `[secrets]` | optional | Master key for secret encryption. **Absent → secret operations error.** |
| `[webhook]` | optional | The webhook ingress listener. **Absent → no webhook server; triggers fire only manually.** |
| `[mail]` | optional | SMTP for invitations. **Absent → a no-op mailer.** |
| `[oauth]` | optional | GitHub OAuth credentials. **Absent → OAuth login disabled.** |

> The two sample files (`local.toml`, `docker.toml`) only fill the always-on
> sections. To enable secrets, webhooks, email, or OAuth in a real deployment,
> add the corresponding optional table — see the
> [reference](../reference/configuration.md) for the exact keys.

## Environment overrides

Secrets don't belong in an image, so the compose stack overrides sensitive values
from the environment:

- **`DATABASE_URL`** — overrides `[database].url`. The compose file builds it from
  `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` so the database URL never
  has to be committed.
- **`RUST_LOG`** — log verbosity (e.g. `info`, `debug`, `scylla_core=debug`).

Inject the secret master key and any OAuth/SMTP credentials the same way at
deploy time rather than committing them.

# Configuration reference

Every configuration knob in one place: the control-plane TOML, the environment
overrides, and the agent CLI. For the *why* behind these, see
[Configuration](../operating/configuration.md).

## Control-plane config (TOML)

Selected with `--config <file>`. Sections marked *optional* disable a feature when
omitted; the rest are always present (defaulted if absent).

### `[grpc]`

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `address` | socket addr | `127.0.0.1:50051` | gRPC / gRPC-Web listen address (bind `0.0.0.0:50051` to accept remote agents). |

### `[database]`

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `url` | string | — | PostgreSQL connection URL (usually overridden by `DATABASE_URL`). |
| `max_connections` | int | — | Pool upper bound (prod sample: 32). |
| `min_connections` | int | — | Pool lower bound (prod sample: 4). |
| `acquire_timeout` | duration | — | Max wait to acquire a connection, e.g. `"30s"`. |
| `run_migrations` | bool | — | Apply pending migrations at boot. Shipped configs: `true`. |

### `[cors]`

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `allow_origins` | \[string\] | `["*"]` | Allowed browser origins. **Pin in production.** |
| `allow_methods` | \[string\] | `GET, POST, PUT, DELETE, OPTIONS` | Allowed HTTP methods. |
| `allow_headers` | \[string\] | `content-type, authorization, x-grpc-web, x-user-agent` | Allowed request headers. |
| `expose_headers` | \[string\] | `grpc-status, grpc-message, grpc-status-details-bin` | Response headers exposed to the browser. |
| `max_age_seconds` | int | `600` | Preflight cache lifetime. |

### `[bootstrap]`

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `username` | string | `admin` | First-boot admin username. |
| `password` | string | `admin123` | First-boot admin password. **Change it.** |
| `email` | string | *(none)* | Optional admin email (enables email login for it). |

### `[metering]`

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `max_projects_per_org` | int | `100` | Per-organization project quota, enforced on creation. |

### `[secrets]` — *optional*

Absent → the secret store is disabled and secret operations error.

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `master_key` | string | — | AEAD master key, **64 hex chars (32 bytes)**. Inject at deploy time. |

### `[webhook]` — *optional*

Absent → no webhook server starts; webhook triggers can only be fired manually.

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `address` | socket addr | — | Webhook HTTP bind address, e.g. `0.0.0.0:8088`. |
| `public_base_url` | string | *(none)* | Public base advertised in `TriggerView.webhook_url`, e.g. `https://hooks.example.com`. Absent → `webhook_url` left empty. |

### `[mail]` — *optional*

Absent → a no-op mailer (invitations aren't emailed).

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `host` | string | — | SMTP host. |
| `port` | int | `465` | SMTP port. |
| `username` | string | — | SMTP username. |
| `password` | string | — | SMTP password. |
| `from` | string | — | Sender, e.g. `"Scylla <no-reply@scylla.dev>"`. |

### `[oauth.github]` — *optional*

Absent → OAuth login is not registered.

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `client_id` | string | — | GitHub OAuth app client id. |
| `client_secret` | string | — | GitHub OAuth app client secret. |
| `redirect_uri` | string | — | OAuth redirect URI. |

## Environment variables

| Variable | Applies to | Meaning |
|----------|-----------|---------|
| `DATABASE_URL` | control plane | Overrides `[database].url` (keeps credentials out of the image). |
| `RUST_LOG` | control plane, agent | Log verbosity, e.g. `info`, `debug`, `scylla_core=debug`. |
| `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` | compose | Compose the DB name/credentials + `DATABASE_URL`. |
| `VITE_API_URL` | frontend build | API URL baked into the UI at **build** time. |
| `VERSION` | compose / release | Image tag to pull / push. |
| `DOCKER_USER` | compose / release | Docker Hub namespace for images. |

## Agent CLI flags

Every flag has an environment-variable equivalent.

| Flag | Env | Default | Meaning |
|------|-----|---------|---------|
| `--control-plane-url` | `SCYLLA_CONTROL_PLANE_URL` | `http://127.0.0.1:50051` | Control-plane gRPC URL. |
| `--app-id` | `SCYLLA_APP_ID` | — | App identity to authenticate as (required). |
| `--app-secret` | `SCYLLA_APP_SECRET` | — | App secret, exchanged for a bearer token (required). |
| `--workspace-root` | `SCYLLA_WORKSPACE_ROOT` | `/var/lib/scylla/workspaces` | Parent dir; each job gets `<root>/<job-id>`. |
| `--keep-workspace` | `SCYLLA_KEEP_WORKSPACE` | `false` | Keep a job's workspace after it ends (debugging). |
| `--publish-buffer-size` | — | `8192` | In-process channel buffer feeding the up-stream (1–1048576). |
| `--max-reconnect-attempts` | `SCYLLA_MAX_RECONNECT_ATTEMPTS` | `10` | Consecutive failed reconnects before exit (`0` = forever). |
| `--reconnect-backoff-secs` | `SCYLLA_RECONNECT_BACKOFF_SECS` | `3` | Base delay between reconnect attempts (doubles per failure, capped at 60s). |

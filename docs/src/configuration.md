# Configuration

## Control plane: `scylla-ce`

### Command line

| Flag                              | Meaning                                                        |
| --------------------------------- | -------------------------------------------------------------- |
| `-c`, `--config <path>`           | The TOML configuration file. Without it, the server uses the defaults below. |
| `-e`, `--print-example-config`    | Print a configuration with the default values, then exit.      |
| `--no-ui`                         | Serve the API without the web UI. Overrides `[ui].enabled`.    |

### Environment variables

| Variable            | Meaning                                                              |
| ------------------- | -------------------------------------------------------------------- |
| `SCYLLA_MASTER_KEY` | Replaces `[secrets].master_key`. An empty value is ignored.          |
| `RUST_LOG`          | The log filter. Without it, the server logs at `info` and hides the audit lines of granted checks; denials stay visible. `RUST_LOG=audit=info` shows every check. |

`scylla-ce` reads no other variable. In particular it does not read
`DATABASE_URL`: set the database URL in `[database].url`.

### Shipped files

`binaries/scylla-ce/config/` holds three files:

| File          | Use                                                                  |
| ------------- | -------------------------------------------------------------------- |
| `docker.toml` | The compose stack. Development values, including a public master key and the default admin password. |
| `local.toml`  | A native development build, with the UI served by the scylla-web dev server. |
| `prod.toml`   | A starting point for production. It has no `[bootstrap]` and no `[secrets]` section. |

### `[server]`

| Key        | Default          | Meaning                                                        |
| ---------- | ---------------- | -------------------------------------------------------------- |
| `address`  | `127.0.0.1:8080` | The one listener: UI, gRPC, gRPC-Web, reflection and webhooks. Required when the section is present. Use `0.0.0.0:8080` in a container. |

The section may also be named `[grpc]`.

### `[server.tls]`

Optional. Without it the server speaks plain HTTP, and a reverse proxy in
front terminates TLS.

| Key    | Meaning                         |
| ------ | ------------------------------- |
| `cert` | Path of the PEM certificate chain. |
| `key`  | Path of the PEM private key.    |

With TLS, the server offers HTTP/2 and HTTP/1.1 over ALPN, so browsers, gRPC
clients and webhook senders all use the same socket.

### `[ui]`

| Key       | Default | Meaning                                                            |
| --------- | ------- | ------------------------------------------------------------------ |
| `enabled` | `true`  | Serve the web UI.                                                  |
| `dir`     | none    | Serve the UI from this directory (a built `dist/`) instead of the copy compiled into the binary. |

### `[database]`

| Key               | Default                                            | Meaning                                    |
| ----------------- | -------------------------------------------------- | ------------------------------------------ |
| `url`             | `postgres://scylla:scylla@localhost:5432/scylla`   | PostgreSQL connection URL. Required when the section is present. |
| `max_connections` | `16`                                               | Pool size.                                 |
| `min_connections` | `1`                                                | Connections kept open.                     |
| `acquire_timeout` | `30s`                                              | Wait for a free connection, as a duration (`"30s"`). |
| `run_migrations`  | `false` when the section is present, `true` when it is absent | Apply the embedded migrations at start. |

Set `run_migrations = true` explicitly, as the shipped files do.

### `[bootstrap]`

The initial administrator. Optional: without this section no account is
created at start.

| Key        | Meaning                                    |
| ---------- | ------------------------------------------ |
| `username` | Required.                                  |
| `password` | Required. 8 to 255 characters.             |
| `email`    | Optional.                                  |

At each start the server creates this user if it does not exist, and makes
sure that it holds the System Admin role. It does not change the password of
an existing user. The server logs a warning when the credentials are
`admin` / `admin123`.

### `[secrets]`

| Key          | Meaning                                                         |
| ------------ | --------------------------------------------------------------- |
| `master_key` | 64 hex characters (32 bytes). Encrypts project secrets and webhook signing secrets. |

`SCYLLA_MASTER_KEY` overrides it. A key that is not 64 hex characters stops
the server at start. Without a key, the server starts, but `CreateSecret` and
the creation of a webhook trigger fail.

`docker.toml` and `local.toml` hold a public development key. The server logs
an error at start when it uses that key. Generate a real one with
`openssl rand -hex 32`. Keep it: the secrets stored with one key cannot be
read with another.

### `[webhook]`

| Key               | Meaning                                                       |
| ----------------- | ------------------------------------------------------------- |
| `public_base_url` | The public origin of the instance, for example `https://ci.example.com`. The server shows `<public_base_url>/webhooks/<trigger_id>` as the URL of a webhook trigger. |

Without it the URL of a webhook trigger is empty in the UI and the API. The
ingress still answers on `/webhooks/{trigger_id}`.

### `[cors]`

The UI is served from the same origin as the API and needs no CORS. This
section is for a UI dev server on another origin, and for third-party
browser clients.

| Key               | Default                                                         |
| ----------------- | --------------------------------------------------------------- |
| `allow_origins`   | `["*"]`                                                         |
| `allow_methods`   | `["GET", "POST", "PUT", "DELETE", "OPTIONS"]`                   |
| `allow_headers`   | `["content-type", "authorization", "x-grpc-web", "x-user-agent"]` |
| `expose_headers`  | `["grpc-status", "grpc-message", "grpc-status-details-bin"]`    |
| `max_age_seconds` | `600`                                                           |

`"*"` in `allow_origins` accepts any origin, and the server logs a warning at
start. This is the default when the section is absent. An empty list accepts
no cross-origin request.

### `[mail]`

Optional SMTP settings. The server uses them to send invitation emails.
Without this section it sends no mail.

| Key        | Default | Meaning                 |
| ---------- | ------- | ----------------------- |
| `host`     |         | SMTP host.              |
| `port`     | `465`   | SMTP port.              |
| `username` |         | SMTP user.              |
| `password` |         | SMTP password.          |
| `from`     |         | Sender address.         |

### `[oauth.github]`

Optional. Enables sign-in with GitHub.

| Key             | Meaning                         |
| --------------- | ------------------------------- |
| `client_id`     | The OAuth app client id.        |
| `client_secret` | The OAuth app client secret.    |
| `redirect_uri`  | The callback URL registered on GitHub. |

### Example

```toml
[server]
address = "0.0.0.0:8080"

[webhook]
public_base_url = "https://ci.example.com"

[database]
url = "postgres://scylla:change-me@db.internal:5432/scylla"
max_connections = 32
min_connections = 4
acquire_timeout = "30s"
run_migrations = true

[bootstrap]
username = "admin"
password = "a-long-unique-password"

[cors]
allow_origins = []
```

Pass the master key with `SCYLLA_MASTER_KEY` rather than in the file.

## Agent: `scylla-agent`

Every flag also reads an environment variable, except
`--publish-buffer-size`. A flag on the command line wins over the variable.

| Flag                          | Variable                         | Default                       | Meaning |
| ----------------------------- | -------------------------------- | ----------------------------- | ------- |
| `--control-plane-url`         | `SCYLLA_CONTROL_PLANE_URL`       | `http://127.0.0.1:8080`       | The control plane. `https://` uses TLS with the system root certificates. |
| `--app-id`                    | `SCYLLA_APP_ID`                  | required                      | The id of the agent's App. |
| `--app-secret`                | `SCYLLA_APP_SECRET`              | required                      | A secret of that App. |
| `--workspace-root`            | `SCYLLA_WORKSPACE_ROOT`          | `/var/lib/scylla/workspaces`  | The directory that holds the job workspaces. Must be writable. |
| `--keep-workspace`            | `SCYLLA_KEEP_WORKSPACE`          | `false`                       | Keep each job workspace after the job ends, to debug. |
| `--max-reconnect-attempts`    | `SCYLLA_MAX_RECONNECT_ATTEMPTS`  | `10`                          | Failed connections in a row before the agent exits. `0` retries forever. |
| `--reconnect-backoff-secs`    | `SCYLLA_RECONNECT_BACKOFF_SECS`  | `3`                           | The base delay between attempts, at least 1. |
| `--publish-buffer-size`       |                                  | `8192`                        | Messages buffered for the control plane, 1 to 1048576. |

`RUST_LOG` sets the log filter of the agent; the default is `info`.

Prefer the variables for the secret: a command line is visible to the other
users of the machine.

See [Running an agent](./running-an-agent.md) for what these settings do.

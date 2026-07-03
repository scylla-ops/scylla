# Operations & troubleshooting

Day-two operations and fixes for the failures you're most likely to hit.

## Health & status

```sh
just status          # docker compose ps — is everything up & healthy?
just logs            # follow all services
just logs scylla-control-plane   # or one service
```

Postgres has a `pg_isready` healthcheck and the frontend a `/healthz` probe, so
`ps` shows `healthy` once each is truly ready — the control plane waits for
Postgres to be healthy before starting.

## Common problems

**Port already in use.** Something else holds `8080`, `5432`, `50051`, or `8088`.
Stop it, or change the host port mapping in `docker-compose.yaml`.

**Control plane can't reach Postgres.** Confirm `postgres` is `healthy`
(`just status`). If it's stuck, reset the volume with `just clean` (wipes data)
and bring the stack back up.

**Frontend shows gRPC / CORS errors.** The browser must reach the control plane at
the `VITE_API_URL` baked into the frontend build, and that origin must be allowed
by `[cors].allow_origins`. The default config already allows
`http://localhost:8080`; if you changed origins, update CORS to match. Remember
`VITE_API_URL` is build-time — a wrong value means rebuilding the frontend image.

**Agent not picking up jobs.** Agents run out-of-band, not in the compose stack.
Check the agent's own logs and confirm it can reach the control plane at its
`--control-plane-url` with a valid `--app-id` / `--app-secret`. In the UI the App
shows **connected** once its worker stream is open; only connected agents receive
work.

## Orphaned jobs

A running job whose agent disconnects (crash, network drop, or a stop without a
clean shutdown) transitions to **`Orphaned`** — a terminal state meaning "this job
lost its worker and we can't know how it ended". If you see orphaned jobs:

- Check why the agent dropped (its logs, host health, network).
- Re-run the pipeline once a healthy agent is connected. A new run is a new job.

Because presence is the live stream (no heartbeats), a disconnect is detected when
the stream closes.

## Logs & verbosity

Set `RUST_LOG` on the control plane (and agent) to raise detail:

```sh
RUST_LOG=info                    # default in the compose stack
RUST_LOG=debug                   # everything
RUST_LOG=scylla_core=debug,info  # one crate louder than the rest
```

## Resetting

- `just down` — stop the stack, keep data.
- `just clean` — stop **and wipe** volumes + locally-built images (destructive;
  use when a beta upgrade is incompatible).
- `just db-reset` — drop and recreate only the Postgres dev volume.

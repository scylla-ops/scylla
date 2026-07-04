# Operations & troubleshooting

Day-two operations and the failures you're most likely to hit.

## Health & status

```sh
just status          # docker compose ps — is everything up & healthy?
just logs            # follow all services
just logs scylla-control-plane   # or one service
```

Postgres has a `pg_isready` healthcheck and the frontend a `/healthz` probe;
the control plane waits for Postgres to be healthy before starting.

## Common problems

**Port already in use.** Something else holds `8080`, `5432`, `50051`, or
`8088`. Stop it, or change the host port mapping in `docker-compose.yaml`.

**Control plane can't reach Postgres.** Confirm `postgres` is `healthy`
(`just status`). If it's stuck, reset the volume with `just clean` (wipes
data) and bring the stack back up.

**Frontend shows gRPC / CORS errors.** The browser must reach the control
plane at the `VITE_API_URL` baked into the frontend build, and that origin
must be allowed by `[cors].allow_origins` (the default config allows
`http://localhost:8080`). `VITE_API_URL` is build-time — a wrong value means
rebuilding the frontend image.

**Agent not picking up jobs.** Agents run out-of-band, not in the compose
stack. Check the agent's own logs and confirm it can reach the control plane
at its `--control-plane-url` with a valid `--app-id` / `--app-secret`. Only
agents shown as **connected** in the UI receive work.

## Orphaned jobs

A running job whose agent disconnects (crash, network drop, unclean stop)
becomes **`Orphaned`** — terminal: the job lost its worker and the outcome is
unknown. Check why the agent dropped (its logs, host health, network), then
re-run the pipeline once a healthy agent is connected — a new run is a new
job. Presence is the live stream (no heartbeats), so a disconnect is detected
when the stream closes.

## Logs & verbosity

Set `RUST_LOG` on the control plane (and agent):

```sh
RUST_LOG=info                    # default in the compose stack
RUST_LOG=debug                   # everything
RUST_LOG=scylla_core=debug,info  # one crate louder than the rest
```

## Resetting

- `just down` — stop the stack, keep data.
- `just clean` — stop **and wipe** volumes + locally-built images
  (destructive; use when a beta upgrade is incompatible).
- `just db-reset` — drop and recreate only the Postgres dev volume.

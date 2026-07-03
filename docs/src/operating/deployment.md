# Deployment

> 🚧 *Chapter in progress.*

Run Scylla in production with Docker Compose and the published multi-arch images.

## The compose stack

<!-- postgres, scylla-control-plane (50051 + 8088), scylla-frontend (8080). Agents out-of-band. -->

## Images & tags

<!-- Multi-arch amd64/arm64; VERSION tag; built via `just release`. -->

## Ports & networking

<!-- 8080 UI, 50051 gRPC, 8088 webhooks, 5432 postgres. -->

## Frontend build-time config

<!-- VITE_API_URL baked into assets at build. -->

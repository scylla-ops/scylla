# Frontend

The web UI is a React application built on clean-architecture principles. It has
its **own, more detailed documentation** in the repository — this chapter is a
pointer plus how the UI fits the rest of Scylla.

## Where the full docs live

The frontend maintains its own docs under `apps/frontend/`:

- `apps/frontend/docs/architecture.md` — module structure, the 4-layer feature
  anatomy (domain / infrastructure / presentation / DI), data flow, routing,
  shared patterns, and the full tech stack.
- `apps/frontend/docs/naming-conventions.md` — naming rules.
- `apps/frontend/README.md` — local dev setup (Vite dev server).

This book intentionally does **not** duplicate them; treat those files as the
source of truth for frontend internals.

## How it fits

The UI is a **static build** served by Caddy (`scylla-frontend`, port `8080`). It
holds no server state of its own — everything comes from the control plane over
gRPC-Web:

- It talks to the API with **`@protobuf-ts/grpcweb-transport`**, using clients
  generated from `scylla-protocol` — the same `.proto` the backend uses, so the UI
  and server never drift.
- The API URL is **baked in at build time** via `VITE_API_URL` (see
  [Deployment](../operating/deployment.md)); it is not read at runtime.
- Because the browser calls the control plane directly, the API's
  [CORS configuration](../operating/configuration.md) must allow the UI's origin.

## Tech stack at a glance

React 18 · TypeScript · TanStack Query (server state) · Zustand (UI state) ·
React Router 7 · Lingui (i18n: en/fr) · shadcn/ui + Radix + Tailwind CSS 4 ·
Vite. Full details — including the layered module structure and DI wiring — are in
`apps/frontend/docs/architecture.md`.

# Scylla — agent guide

Repo-wide pointers for agents. Each area keeps its own guide next to the code;
this file only says where to look.

## Repository shape

`binaries/` holds the packages that produce an executable, `crates/` the
libraries both of them link.

| Path | What it is |
|---|---|
| `crates/scylla-domain` | dependency-light shared kernel (domain model, `JobEvent`) |
| `crates/scylla-proto` | the wire contract — protos under `proto/scylla/<domain>/v1/` |
| `binaries/scylla-control-plane` | server binary: web UI, gRPC API, job dispatch, cron, webhooks |
| `binaries/scylla-agent` | the worker installed per machine |
| `apps/frontend` | the web UI's source; compiled into the control-plane binary |

Both binary packages sit exactly two directories below the root, and that depth
is load-bearing: `sqlx::migrate!("../../migrations")` resolves against
`CARGO_MANIFEST_DIR`.

## Guides

| Topic | Where |
|---|---|
| **Publishing release images** | [`RELEASING.md`](./RELEASING.md) |
| Access model (grants, roles, permissions) | [`docs/src/access-model.md`](./docs/src/access-model.md) |
| Domain vocabulary | [`GLOSSARY.md`](./GLOSSARY.md) |
| Running the stack | [`README.md`](./README.md) |
| Frontend architecture and module rules | [`apps/frontend/CLAUDE.md`](./apps/frontend/CLAUDE.md) |
| A frontend module's contract | that module's `AGENTS.md`, e.g. `apps/frontend/src/modules/features/roles/AGENTS.md` |

## Conventions worth knowing before editing

- **Commands live in the `justfile`, logic does not.** `just --list` is the
  index. Recipes stay one-liners that delegate; a recipe growing an `if` or a
  `case` is a sign the logic belongs in the tool it calls.
- **Container builds live in `docker-bake.hcl`.** Tags included: they are
  derived from `VERSION` and `LATEST` in the bake file, not assembled by a
  wrapper. See [`RELEASING.md`](./RELEASING.md).
- **The backend builds offline.** `SQLX_OFFLINE=true` against the committed
  `.sqlx/` cache; after touching a `query!`/`query_as!` macro, run
  `just db-prepare` and commit the result.
- **Protos are linted and breaking-checked.** `just proto-lint`,
  `just proto-fmt`, `just proto-breaking`.
- **The frontend has hard CI gates** beyond typecheck and lint: architecture
  boundaries (`pnpm depcruise`), module cycles, and i18n catalog
  collisions. A change that passes `tsc` can still fail the build.

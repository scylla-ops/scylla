# Scylla Frontend

Welcome to the Scylla frontend repository.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v9+)
- The Scylla backend running (`cargo run -p scylla-ce -- --config ../../binaries/scylla-ce/config/local.toml --no-ui`, port `8080` by default)

### Environment Variables

Copy the example file and adjust if needed:

```bash
cp .env.example .env
```

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_API_URL` | URL of the Scylla gRPC-Web backend. Only needed here: a deployed instance serves this bundle from the control plane itself, leaves the variable unset, and the client uses relative URLs. | `http://localhost:8080` |

### Install & Run

```bash
# Install dependencies
pnpm install

# Start the dev server (includes proto generation + i18n compilation)
pnpm dev
```

The app will be available at `http://localhost:5173`.

### Other Commands

| Command | Description |
|---------|-------------|
| `pnpm build` | Production build |
| `pnpm preview` | Preview production build locally |
| `pnpm typecheck` | Run TypeScript type checking |
| `pnpm lint` | Run ESLint |
| `pnpm gen-proto` | Regenerate gRPC TypeScript clients from proto files |
| `pnpm extract` | Extract i18n translation strings |
| `pnpm compile` | Compile i18n translations |

## Documentation

- [Architecture](docs/architecture.md) — Module structure, data flow, shared patterns, tech stack
- [Naming Conventions](docs/naming-conventions.md) — File, folder, TypeScript & React naming rules
- [CLAUDE.md](CLAUDE.md) — The working contract for this codebase (rules, checklists, conventions)

## Module documentation

The app is built as independent modules in four layers, with dependencies pointing only
downward. Each module has a **README.md** explaining what it does and why it is built that way,
and an **AGENTS.md** — a condensed, rule-focused brief for AI coding agents.

```
app/ (core + layout)   composition root — may import anything
        ↓
features/              the business modules
        ↓
platform/              cross-cutting capabilities — may never import a feature
        ↓
shared/                generic UI + utils, no business meaning — imports nobody
```

### `app/` — the shell

| Module | What it owns |
|--------|--------------|
| [core](src/modules/core/README.md) | Composition root: module registry, router skeleton, auth guard, global error handling |
| [layout](src/modules/layout/README.md) | App shell: sidebar, top bar, breadcrumbs, context selector |

### `features/` — the business modules

| Module | What it owns |
|--------|--------------|
| [agents](src/modules/features/agents/README.md) | Build agents — the machines that pick up jobs — and their run statistics |
| [apps](src/modules/features/apps/README.md) | Machine identities and the secrets they authenticate with |
| [dashboard](src/modules/features/dashboard/README.md) | The organization landing page — a composite view owning no data |
| [jobs](src/modules/features/jobs/README.md) | Pipeline runs: status, logs, and the live tail of both |
| [login](src/modules/features/login/README.md) | Sign-in and the session token |
| [marketplace](src/modules/features/marketplace/README.md) | Ready-made pipeline templates |
| [membership](src/modules/features/membership/README.md) | Who belongs to an organization or project, and with which roles |
| [organization](src/modules/features/organization/README.md) | The top-level tenant and the switcher in the shell |
| [pipeline](src/modules/features/pipeline/README.md) | Pipeline definitions, the visual + script editor, and running them |
| [project](src/modules/features/project/README.md) | The unit that owns pipelines, secrets and members |
| [roles](src/modules/features/roles/README.md) | Role catalog, grants, and the permission vocabulary |
| [secret](src/modules/features/secret/README.md) | Project-scoped secrets injected into runs |
| [triggers](src/modules/features/triggers/README.md) | What starts a pipeline without a human: schedules and webhooks |
| [user](src/modules/features/user/README.md) | User accounts: the directory and per-user settings |

### `platform/` — cross-cutting capabilities

| Module | What it owns |
|--------|--------------|
| [authz](src/modules/platform/authz/README.md) | `Permission`, `useCan`, `Can`, `RequirePermission` — the read side of authorization |
| [context](src/modules/platform/context/README.md) | The active organization / project / pipeline, and navigation derived from it |
| [di](src/modules/platform/di/README.md) | Dependency injection mechanism (the wiring lives in `core`) |
| [grpc](src/modules/platform/grpc/README.md) | The single gRPC-Web transport, with auth attached |
| [routing](src/modules/platform/routing/README.md) | The `ScyllaModule` contract and the route/nav composer |

### `shared/`

| Module | What it owns |
|--------|--------------|
| [shared](src/modules/shared/README.md) | Generic UI, hooks, `ScyllaResult`, shadcn primitives — no business meaning |

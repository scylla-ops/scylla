# Scylla Frontend

Welcome to the Scylla frontend repository.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v9+)
- The Scylla backend running (gRPC server on port `50051` by default)

### Environment Variables

Copy the example file and adjust if needed:

```bash
cp .env.example .env
```

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_API_URL` | URL of the Scylla gRPC-Web backend | `http://localhost:50051` |

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

# Scylla Frontend — CLAUDE.md

React + TypeScript frontend for Scylla, built on **Clean Architecture** with **TanStack Query** for async data. This file is the contract for working in this codebase — follow it. Full references: `docs/architecture.md` and `docs/naming-conventions.md`.

---

## Read the module's `AGENTS.md` first

**Every module has an `AGENTS.md` at its root.** Before writing or changing code in a module,
read that file. It is written for you, and it holds what this document cannot: that module's
exact public API, its repository methods, its routes and permissions, its file map, and the
specific mistakes that module invites.

```
src/modules/features/<feature>/AGENTS.md      the 14 business modules
src/modules/platform/<capability>/AGENTS.md   authz, context, di, grpc, routing
src/modules/core/AGENTS.md                    composition root
src/modules/layout/AGENTS.md                  app shell
src/modules/shared/AGENTS.md                  generic UI + utils
```

- Touching one module → read its `AGENTS.md`.
- Touching several → read each one. Cross-module work is where the rules bite hardest.
- Consuming another module's hook or type → read *its* `AGENTS.md` to find what the barrel
  actually exports, instead of guessing or deep-importing.

This file stays authoritative for anything that spans the whole codebase — layering, naming,
React rules, the commands below. A module's `AGENTS.md` never contradicts it; it makes it
concrete. **If you find a contradiction, this file wins and the `AGENTS.md` is stale — fix it.**

Each module also has a **`README.md`**, written for humans: what the module is for, and the
reasoning behind how it is built. Read it when you need the *why*; `AGENTS.md` gives you the
*what*. The root [`README.md`](README.md) links to all of them.

**Keep both current.** Changing a module's public API, routes, permissions, repository contract
or structure means updating its `AGENTS.md` in the same change — and its `README.md` too when
the reasoning changed, not just the code.

---

## Commands

Package manager is **pnpm** (`pnpm@11.1.2`). Run all commands from `apps/frontend/`.

| Command | Purpose |
|---------|---------|
| `pnpm dev` | Run prebuild + Vite dev server |
| `pnpm build` | prebuild + typecheck + production build |
| `pnpm typecheck` | `tsc -b` — type check only |
| `pnpm lint` | ESLint, `--max-warnings 0` (warnings are errors) |
| `pnpm lint:fix` | ESLint with `--fix` |
| `pnpm gen-proto` | Generate gRPC/protobuf-ts clients |
| `pnpm extract` / `pnpm compile` | Lingui: extract messages / compile catalogs |
| `pnpm depcruise` | Architecture rules: layer direction **and** public-API surface — **must be clean** |
| `pnpm depcruise:cycles` | Module dependency cycles — **must be zero** |

> `depcruise:cycles` is **not** redundant with `depcruise`'s `no-circular`. That rule works on the
> *file* graph; a cycle can exist between two folders with no file in a loop (`a/x.ts → b/index.ts`,
> `b/y.ts → a/index.ts`, where `b/index.ts` never reaches `y.ts`). `scripts/check-module-cycles.mjs`
> collapses the graph to module granularity and runs Tarjan, which is what catches those. Verified:
> `no-circular` misses that case, the script reports it. Keep both.

`prebuild` = `gen-proto` + `extract` + `compile`, and runs before `dev` and `build`. **Do not hand-edit generated proto code or compiled locale `messages.ts` files** — regenerate them.

Before considering work done: `pnpm typecheck`, `pnpm lint`, `pnpm depcruise` and
`pnpm depcruise:cycles` must all pass clean (zero warnings, zero violations, zero cycles).
CI runs all four.

**After moving any component between modules**, run `node scripts/restore-translations.mjs` —
Lingui catalogs are per-module and keyed by source string, so `extract` silently drops the
French translation of anything that moved. Confirm with `--dry-run` that nothing is left.

---

## Architectural Rules (non-negotiable)

The app has **four layers, and dependencies only ever point down**:

```
app/  (core/ + layout/)   composition root — router shell, providers, module registry.  May import anything.
  ↓
features/                 the business modules.  May import platform + shared, never the shell,
                          and never another feature's internals.
  ↓
platform/                 cross-cutting capabilities that own a domain: authz, context, di,
                          grpc, routing.  MUST NEVER import a feature.
  ↓
shared/                   generic UI + utils with no business meaning.
```

This is machine-enforced (`.dependency-cruiser.cjs`), not a convention. **The module graph is
cycle-free and must stay that way** — `pnpm depcruise:cycles` is a CI gate.

### Every module is reached through its `index.ts` — nothing else

A module's `index.ts` **is** its public API. Everything else in it is private and free to move.

```typescript
import { useProjects } from '@/modules/features/project';   // ✅ the public API
import { useProjects } from '@/modules/features/project/presentation/hooks/useProjects.ts'; // ❌
import { Permission } from '@platform/authz';               // ✅
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';           // ❌
```

Enforced by six rules in `.dependency-cruiser.cjs`, all `error`: `feature-api-only`,
`shell-uses-feature-api`, `platform-api-only`, `platform-capability-api-only`,
`module-declaration-is-private`, `domain-accessor-is-private`.

What a barrel must **not** export:
- **`<feature>.module.ts`** — it instantiates the feature's data sources at import time, so
  re-exporting it pulls that feature's gRPC client into the chunk of anyone who imports the
  barrel. The registry imports it directly by path; that is the only door.
- **`use-<feature>-domain.ts`** — a feature pins the type of its *own* injection there. Another
  module calling it queries your repository behind your hooks' back and forks the query cache
  into two keys for one resource.
- **Pages**, as a rule. Two exceptions exist and are marked as such (`JobsPage`,
  `UserSettingsPage`): another module composes them behind its own route, and both consumers
  are themselves lazily loaded.

Need something from another feature that its `index.ts` does not export? Add the export **to
that feature**, or ask it for a hook that does the job. Do not reach past the barrel — a
"temporary" deep import is how the sixteen modules once became one.

Inside a feature, dependencies point **inward**: `presentation → domain ← infrastructure`.

```
feature/
├── feature.module.ts    → THE module declaration: { id, domain, routes?, nav? }
│                          private: only core/di/registry.ts imports it
├── index.ts             → public API — the ONLY thing other modules may import (enforced)
├── domain/              → PURE business logic, ZERO external deps (no React, no gRPC, no proto)
│   ├── repository/      → repository INTERFACES + their input types (the data contract)
│   ├── entities/        → {Name}Entity — identity-bearing business objects (*.entity.ts)
│   ├── structs/         → value objects, enums, DTOs, wrappers (*.struct.ts)
│   └── use-cases/       → OPTIONAL — only when one earns its place (see below)
├── infrastructure/      → concrete implementations of domain abstractions
│   ├── repository/
│   │   ├── default-feature.repository.ts   → repository impl
│   │   ├── data-sources/                    → data source interface
│   │   └── mappers/                         → proto ↔ domain mapping
│   └── data/remote/                         → gRPC call impl
├── locales/             → i18n (en/, fr/)
└── presentation/        → UI layer
    ├── hooks/           → TanStack Query hooks + the module's domain accessor
    ├── stores/          → Zustand stores (ephemeral UI state ONLY)
    └── ui/              → React components (pages, dialogs, tables, columns)
```

### Layer responsibilities

- **Domain** — The repository interface *is* the module's data contract, and presentation calls it
  directly. Repository interfaces know nothing about gRPC/HTTP, and their input types live beside
  them (e.g. `CreateGrantInput` in `permission.repository.ts`). No framework imports here, ever.
  Split domain types into two buckets, both proto-independent: **entities** (`entities/*.entity.ts` → `{Name}Entity`) are the identity-bearing things the feature owns (e.g. `SecretEntity`, `RoleEntity`, `UserEntity`); they may hold related input shapes (`CreateSecretInput`) and pure behavior (`updateRole`). **Structs** (`structs/*.struct.ts`) are plain data shapes with no identity — value objects, enums, DTOs, and list/result wrappers (e.g. `Permission`, `PermissionScope`, `ProjectList`, `CreatedApp`) — named with plain PascalCase (no suffix). The `Entity` suffix is what distinguishes the two at every use site, including against the proto types (which keep the bare name, aliased in mappers). There are **no `*.model.ts` files** — every domain type is an entity or a struct.
- **Infrastructure** — Data sources (interface + `*.impl`) per transport; repository impl coordinates data sources and maps infra types → domain types via mappers (`Grpc{Entity}Mapper`).
- **Presentation** — Hooks wrap the repository (or a use case) with `useQuery`/`useMutation`. Components are dumb-ish: they consume hooks. Zustand is for UI state only (modals, form values) — **server state lives in TanStack Query, never Zustand**.
- **DI** — `platform/di` provides the mechanism, the app provides the wiring. Each feature has a
  typed accessor in `presentation/hooks/use-<feature>-domain.ts`; hooks call
  `const { <feature>Repository } = use<Feature>Domain()`. The concrete map is assembled in
  `core/di/registry.ts` and injected by `DependenciesProvider`.

### Use cases are optional — most operations don't need one

A use case earns its place **only when it does something a repository method cannot**: orchestrating
several calls, applying a business rule, or combining repositories. A class whose `execute(args)`
just forwards to `repository.method(args)` is a second name for the same operation — delete it and
call the repository.

This is not a loophole in the layering: the repository *interface* is domain, so
`presentation → domain ← infrastructure` and the dependency inversion are untouched. It is still
injected, still substitutable in a test. Only the redundant indirection is gone.

> The codebase once had 65 use cases; 64 were pure forwarding. Exactly one survives —
> `UpdateRoleUseCase` (read → apply a pure entity function → save). Use that as the bar.

### Module map

- `core/` — composition root: module registry (`di/registry.ts`), router shell, `App.tsx`, auth guard.
- `platform/` — below the features, may never import one: `authz` (Permission, `useCan`, `Can`,
  `RequirePermission`), `context` (current org/project/pipeline + navigation), `di`, `grpc`, `routing`
  (`ScyllaModule`, route composer, `RouteGuard`).
- `features/` — `agents`, `apps`, `dashboard`, `jobs`, `login`, `marketplace`, `membership`,
  `organization`, `pipeline`, `project`, `roles`, `secret`, `triggers`, `user`.
- `layout/` — App shell (Layout, AppSidebar, ScyllaBreadcrumbs, context-selector).
- `shared/` — Reusable components/hooks/stores/utils. **No business logic, no feature imports.**

### The `ScyllaModule` contract

Each module declares itself in `<feature>.module.ts` at its root, and the router, sidebar and DI
container are all *derived* from that one declaration — there is no second list to keep in sync:

```typescript
export const SecretModule = {
  id: 'secret',
  domain: { secretRepository },              // the DI surface
  routes: [{                                  // grafted by the shell at its mount point
    mount: 'project', path: 'secrets',
    permission: Permission.LIST_SECRETS,      // read by RouteGuard *and* the sidebar
    breadcrumb: () => ({ label: msg`Secrets` }),
    lazy: async () => ({ Component: (await import('./presentation/ui/Secret.page.tsx')).SecretPage }),
  }],
  nav: [/* sidebar entries */],
} satisfies ScyllaModule;
```

Rules that matter:
- `routes.lazy` is what keeps pages out of the initial chunk. Keep it.
- `permission` is declared **once** and drives both the route guard and the sidebar link.
- Breadcrumb/nav labels are `` msg`…` `` descriptors, not JSX, so the module file stays a `.ts`.
- `core/di/registry.ts` imports `<feature>.module.ts` — **never** `<feature>/index.ts`, whose
  re-exported UI would drag every page back into the entry chunk.

---

## Error Handling — `ScyllaResult<T>`

All async operations return `ScyllaResult<T>` (a Result type), not raw throws.

```typescript
const result = await ScyllaResult.tryAsync(async () => api.call(), 'Error message');
result.fold({ onSuccess: data => ..., onError: err => ... });
const data = result.unwrap(); // or throw on error
```

In mutation/query hooks, call `.unwrap()` inside `mutationFn`/`queryFn` so TanStack Query handles the error. `ScyllaError` extends `Error` with gRPC code extraction. Utilities live in `src/modules/shared/utils/`.

---

## Shared Patterns (reuse these — don't reinvent)

- **Selection**: `useSelection(key)` over a single `useSelectionStore`, keyed by feature. Used by `DataTable` + `FeatureHeader`. No per-feature selection store.
- **List headers**: `FeatureHeader` (count, clear/delete selection, new button).
- **Forms**: declarative `ScyllaForm` from `FormItem[]`; `FormDialog` wraps it in a dialog; `useFormState(items)` manages values/changes/reset/validation.
- **Pagination**: `usePagination()` (local page state merged with server `totalCount`/`totalPages`).
- **Navigation**: `useScyllaNavigate()`.
- **Global state**: only `useContextStore` (current org/project) and `useSelectionStore` are app-wide. Everything else = TanStack Query (server) or local `useState`.
- **Batch loads**: use `useQueries` to avoid N+1 (see jobs-per-pipeline).

### Query keys

Always use **factory functions** so queries and invalidations stay in sync:

```typescript
export const JOBS_QUERY_KEY = (pipelineId: string) =>
  ['jobs', 'pipeline', pipelineId, MAX_JOBS_PER_PIPELINE] as const;
// invalidate: queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(id), exact: true });
```

---

## React — Best Practices

### `useEffect` is a last resort, not the default tool

An effect exists to synchronize with a system **outside** React (browser API, subscription, timer, ReactFlow/CodeMirror instance, URL sync). For everything else there is a better tool:

| Need | ❌ Not an effect | ✅ Instead |
|------|-----------------|-----------|
| Value derived from props/state | `useEffect` + `setState` | Compute during render |
| Server data | `useEffect` + gRPC call | A TanStack Query hook (`presentation/hooks/use-*`) |
| React to a user action | effect watching state | The event handler |
| Reset state when a prop changes | effect + `setState` | A `key` on the component |
| Transform a list before display | effect + mirror state | Derive inline (memoize only if measurably costly) |
| Refetch after a mutation | effect | `invalidateQueries` in `onSuccess` |

- **No mirror state.** What TanStack Query already holds (`data`, `isPending`, `error`) is never copied into `useState` or Zustand. Same for form state owned by `useFormState`, and for the current org/project owned by `useContextStore`.
- **Honest dependencies.** Never empty a dep array to silence `react-hooks/exhaustive-deps` — fix the cause (stabilize the callback, move the value out, or drop the effect entirely).
- **No `setState` cascade inside an effect body.** If an effect's only job is to set state from other state, that state was derivable.
- **Don't read/write `ref.current` during render** — refs are for effects and handlers.
- `useMemo` / `useCallback` only when they solve a real problem (expensive computation, reference passed to a memoized child or a hook dep array) — not by reflex. **React 18, the React Compiler is not enabled**, so memoization is manual but still not free.

### Data access

- **A component never reaches the domain directly.** `use<Feature>Domain()` belongs in `presentation/hooks/`; UI consumes hooks only. This holds across the codebase today — keep it that way.
- One remote operation = one hook (`use-create-user.ts`, `use-pipelines.ts`), built on a query-key factory.
- Lists go through `DataTable` + `usePagination()`; don't render thousands of unpaginated rows. Row keys must be stable business ids, never array indices.

### Component splitting

- **One component = one responsibility.** Past ~150 lines, or as soon as you have to scroll to follow the JSX, split it.
- Logic moves into a colocalized `use-*` hook in the feature; the page/dialog keeps rendering only.
- Self-contained or repeated JSX blocks become named components (`PipelineActions`, `FeatureHeader`).
- **Never define a component inside another component** — it is recreated on every render and loses its state.
- A component used by ≥ 2 features moves up to `shared/presentation/ui/` (and gets exported from the relevant barrel `index.ts`).
- Components use **named exports**; a handful of pages use `export default` — match the neighbouring files rather than converting them.

### Minimalism (applies inside the layers, not against them)

The 4-layer structure is mandatory; everything else must earn its place. Before adding a file, an abstraction, a store, or a dependency, ask: **what breaks if I don't?** If the answer is "nothing yet", don't.

- No speculative abstraction. Factor at the 2nd or 3rd real usage, never "just in case". A helper used once stays inline.
- Derive rather than store — an extra piece of state is an extra thing to keep in sync.
- Dead code is deleted, not commented out "for later" — git keeps the history.
- A new dependency must justify itself; 20 lines of local code is often the better trade.

---

## Naming Conventions (enforced)

**Files/folders: kebab-case. React component files: PascalCase.**

| Thing | Pattern | Example |
|-------|---------|---------|
| Page component file | `*.page.tsx` | `UserAdmin.page.tsx` |
| Use case (only when it orchestrates) | `*.use-case.ts` → `{Verb}{Entity}UseCase` | `update-role.use-case.ts` / `UpdateRoleUseCase` |
| Repository interface | `*.repository.ts` (domain) → `{Entity}Repository` | `user.repository.ts` / `UserRepository` |
| Repository impl | `default-*.repository.ts` (infra) → `Default{Entity}Repository` | `default-user.repository.ts` / `DefaultUserRepository` |
| Data source iface | `*.data-source.ts` | `user-remote.data-source.ts` |
| Data source impl | `*.data-source.impl.ts` | `user-remote.data-source.impl.ts` |
| Mapper | `grpc-*.mapper.ts` → `Grpc{Entity}Mapper` | `grpc-user.mapper.ts` |
| Domain entity | `*.entity.ts` → `{Name}Entity` | `secret.entity.ts` / `SecretEntity` |
| Struct (value object / enum / DTO / wrapper) | `*.struct.ts` (plain name, no suffix) | `permission.struct.ts` / `Permission`, `pagination.struct.ts` / `PaginationInfo` |
| Hook | `use-{name}.ts` | `use-create-user.ts` |
| Zustand store | `use-{name}.store.ts` → `use{Name}Store` | `use-context.store.ts` |
| Module declaration (at module root) | `*.module.ts` → `{Feature}Module` | `user.module.ts` / `UserModule` |
| Module domain accessor | `use-{feature}-domain.ts` | `use-user-domain.ts` / `useUserDomain` |
| Module public API | `index.ts` | `features/membership/index.ts` |
| Guard / Wrapper | `*.guard.tsx` / `*.wrapper.tsx` | `Auth.guard.tsx` |
| Router | `*.router.tsx` | `Core.router.tsx` |

Code identifiers: Interfaces/Types/Classes/Components/Enums **PascalCase** (no `I` prefix); hooks **camelCase `use*`**; true constants **UPPER_SNAKE_CASE** (`DEFAULT_PAGE_SIZE`); query-key factories **UPPER_SNAKE_CASE**; props type = `{Component}Props`.

### `interface` vs `type`

- **`interface`** for object shapes that are extended or implemented: component props (`interface UserTableProps`), repository contracts, data source contracts. This is the codebase majority — follow it.
- **`type`** for everything a shape can't express: unions, literal unions, mapped/utility types, function types, aliases (`type PermissionScope = 'org' | 'project'`).
- Type-only imports must use `import type` (`@typescript-eslint/consistent-type-imports` is an error, and `verbatimModuleSyntax` requires it).
- Colocate a type with the file that uses it; a dedicated file is only justified for shared or bulky types — and in `domain/` it must land in `entities/` or `structs/` per the rules above.

> Note: docs/naming-conventions.md shows `{Entity}RepositoryImpl` for the impl, but the actual codebase uses `Default{Entity}Repository` in `default-{entity}.repository.ts`. **Follow the codebase convention.**

---

## i18n (Lingui)

- Use `<Trans>...</Trans>` in JSX and `` t`...` `` for strings. `FormItem` labels accept `ReactNode`.
- Catalogs live in `locales/{en,fr}/messages.po` per feature (+ global). Run `pnpm extract` after adding strings, `pnpm compile` to build catalogs. Don't edit `messages.ts` by hand.

---

## Conventions & tooling

- Path alias `@/` → `src/` (e.g. `@/modules/features/user/...`).
- Prettier: semicolons, single quotes (incl. JSX), 2-space tabs, trailing commas (all), printWidth 100, `arrowParens: avoid`. Match this style; don't reformat unrelated code.
- Routing: React Router v7, centralized in `Core.router.tsx`. Protected routes wrapped by `AuthGuard` + `Layout`.
- Backend comms: gRPC-Web via protobuf-ts through `CoreGrpcTransport`.
- Comments: **English**, concise, and oriented on *why* rather than *what*. Don't narrate code that already reads clearly.
- Lint rules worth knowing (see `eslint.config.js`): `no-floating-promises` and `no-misused-promises` are errors — never fire-and-forget a promise; unused bindings must be prefixed `_` to be tolerated. The `no-unsafe-*` rules are off only because of the generated proto layer — that is not a licence to spread `any`.

### Stack
React 18 · TypeScript 5.8 · TanStack Query 5 · Zustand 5 · React Router 7 · Lingui 5 · gRPC-Web (protobuf-ts) · shadcn/ui + Radix · Tailwind CSS 4 · Vite 7 · Framer Motion.

---

## Adding a feature (checklist)

1. Create `src/modules/features/<feature>/` with `domain/ infrastructure/ locales/ presentation/`.
2. Domain: repository interface (+ its input types), `entities/*.entity.ts` and `structs/*.struct.ts`.
   **Add a use case only if it orchestrates** — see "Use cases are optional".
3. Infrastructure: data source (iface + `.impl`), `default-<feature>.repository.ts`, `grpc-<feature>.mapper.ts`.
4. `<feature>.module.ts` at the module root: `{ id, domain, routes?, nav? } satisfies ScyllaModule`,
   using `grpcTransport` from `@platform/grpc`. Register it in `core/di/registry.ts`.
5. `presentation/hooks/use-<feature>-domain.ts`: `useModuleDomain<typeof XModule.domain>('<id>')`.
6. Presentation: TanStack Query hooks (use query-key factories), Zustand store only if UI state needs it,
   UI components — the domain accessor belongs in hooks only, and no effect where a derivation or a
   handler would do.
7. Declare routes/nav on the module (not in the router); add `locales/{en,fr}/` and register the
   catalog in `lingui.config.js`.
8. `index.ts`: export only what other modules may use — never the `*.module.ts`, never
   `use-<feature>-domain.ts`, and a page only when another module composes it behind its own route.
   Every feature has one, even when nothing consumes it yet: that is where a contributor looks first.
9. Reuse before adding: check `shared/` and the Shared Patterns section first.
10. `AGENTS.md` + `README.md` at the module root, and a row in the root `README.md`'s module
    table. Follow the shape of a neighbouring module's pair: `AGENTS.md` = public API, data
    contract, file map, routes/nav, the rules that bite there; `README.md` = what it is for and
    why it is built that way.
11. `pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` all clean.

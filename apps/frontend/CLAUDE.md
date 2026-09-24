# Scylla Frontend — CLAUDE.md

Svelte 5 + TypeScript frontend for Scylla, built on **Clean Architecture** with **TanStack Query** for async data. This file is the contract for working in this codebase — follow it. Full references: `docs/architecture.md` and `docs/naming-conventions.md`.

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
- Consuming another module's query or type → read *its* `AGENTS.md` to find what the barrel
  actually exports, instead of guessing or deep-importing.

This file stays authoritative for anything that spans the whole codebase — layering, naming,
Svelte rules, the commands below. A module's `AGENTS.md` never contradicts it; it makes it
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
| `pnpm typecheck` | `tsc -b && svelte-check` — `tsc` cannot see `.svelte` files, `svelte-check` can |
| `pnpm test` | Vitest, single run — **must be green** |
| `pnpm test:watch` | Vitest in watch mode |
| `pnpm coverage` | Vitest + v8 coverage, enforces the thresholds in `vite.config.ts` |
| `pnpm lint` | ESLint, `--max-warnings 0` (warnings are errors). Slow (~20 min) and needs an 8 GB heap, which the script sets |
| `pnpm lint:fix` | ESLint with `--fix` |
| `pnpm gen-proto` | Generate gRPC/protobuf-ts clients |
| `pnpm extract` / `pnpm compile` | Lingui: extract messages (`--clean`, drops obsolete entries) / compile catalogs |
| `pnpm depcruise` | Architecture rules: layer direction **and** public-API surface — **must be clean** |
| `pnpm depcruise:cycles` | Module dependency cycles — **must be zero** |
| `pnpm i18n:collisions` | Same msgid translated differently in two catalogs — **must be zero** |

> `depcruise:cycles` is **not** redundant with `depcruise`'s `no-circular`. That rule works on the
> *file* graph; a cycle can exist between two folders with no file in a loop (`a/x.ts → b/index.ts`,
> `b/y.ts → a/index.ts`, where `b/index.ts` never reaches `y.ts`). `scripts/check-module-cycles.mjs`
> collapses the graph to module granularity and runs Tarjan, which is what catches those. Verified:
> `no-circular` misses that case, the script reports it. Keep both.

`prebuild` = `gen-proto` + `extract` + `compile`, and runs before `dev` and `build`. **Do not hand-edit generated proto code or compiled locale `messages.ts` files** — regenerate them.

Before considering work done: `pnpm typecheck`, `pnpm test`, `pnpm lint`, `pnpm depcruise`,
`pnpm depcruise:cycles` and `pnpm i18n:collisions` must all pass clean (zero warnings, zero
failures, zero violations, zero cycles, zero collisions). CI runs all six.

**After moving any component between modules**, run `node scripts/restore-translations.mjs` —
Lingui catalogs are per-module and keyed by source string, so `extract` silently drops the
French translation of anything that moved. Confirm with `--dry-run` that nothing is left.

---

## Architectural Rules (non-negotiable)

The app has **four layers, and dependencies only ever point down**:

```
app/  (core/ + layout/)   composition root — router shell, app shell, module registry.  May import anything.
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
import { projectQueries } from '@/modules/features/project';                                   // ✅ the public API
import { projectQueries } from '@/modules/features/project/presentation/project.queries.ts';   // ❌
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
- **A Svelte component.** Rollup cannot drop a component that a barrel re-exports, so every
  module that imports the barrel would pull the component and its UI library into its chunk.
  Export a **loader** instead: `export const loadJobsPage = () => import('./…/Jobs.page.svelte')`.
  The consumer writes `{#await loadJobsPage() then { default: JobsPage }}`.
- **Pages**, as a rule. Two exceptions exist, as loaders (`loadJobsPage`,
  `loadUserSettingsPage`): another module composes them behind its own route.

Need something from another feature that its `index.ts` does not export? Add the export **to
that feature**, or ask it for a query that does the job. Do not reach past the barrel — a
"temporary" deep import is how the sixteen modules once became one.

Inside a feature, dependencies point **inward**: `presentation → domain ← infrastructure`.

```
feature/
├── feature.module.ts    → THE module declaration: { id, domain, routes? }
│                          private: only core/di/registry.ts imports it
├── index.ts             → public API — the ONLY thing other modules may import (enforced)
├── domain/              → PURE business logic, ZERO external deps (no Svelte, no gRPC, no proto)
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
    ├── <feature>.queries.ts        → TanStack Query options factories (plain TS) + key factories
    ├── <view>.state.svelte.ts      → ViewModels of the views that need one (runes)
    ├── *.actions.ts / *.calculator.ts → Svelte actions / pure algorithms, when needed
    └── ui/              → Svelte components (pages, dialogs, tables) + *.messages.ts
```

### Layer responsibilities

- **Domain** — The repository interface *is* the module's data contract, and presentation calls it
  directly. Repository interfaces know nothing about gRPC/HTTP, and their input types live beside
  them (e.g. `CreateGrantInput` in `permission.repository.ts`). No framework imports here, ever.
  Split domain types into two buckets, both proto-independent: **entities** (`entities/*.entity.ts` → `{Name}Entity`) are the identity-bearing things the feature owns (e.g. `SecretEntity`, `RoleEntity`, `UserEntity`); they may hold related input shapes (`CreateSecretInput`) and pure behavior (`updateRole`). **Structs** (`structs/*.struct.ts`) are plain data shapes with no identity — value objects, enums, DTOs, and list/result wrappers (e.g. `Permission`, `PermissionScope`, `ProjectList`, `CreatedApp`) — named with plain PascalCase (no suffix). The `Entity` suffix is what distinguishes the two at every use site, including against the proto types (which keep the bare name, aliased in mappers). There are **no `*.model.ts` files** — every domain type is an entity or a struct.
- **Infrastructure** — Data sources (interface + `*.impl`) per transport; repository impl coordinates data sources and maps infra types → domain types via mappers (`Grpc{Entity}Mapper`).
- **Presentation** — `*.queries.ts` wraps the repository (or a use case) in `queryOptions` /
  `mutationOptions`. A component or a ViewModel runs them with `createQuery` / `createMutation`
  from `@platform/query`. Components are dumb-ish: they render state. **Server state lives in
  TanStack Query, never in a store.**
- **DI** — `platform/di` provides the mechanism, the app provides the wiring. A feature's
  `*.queries.ts` reads its own repository with
  `getModuleDomain<typeof XModule.domain>('<id>').xRepository`. The concrete map is assembled in
  `core/di/registry.ts` and installed with `setDependencyRegistry`.

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

- `core/` — composition root: module registry (`di/registry.ts`), router shell (`core.router.ts`),
  `App.svelte`, auth guard.
- `platform/` — below the features, may never import one: `authz` (Permission, `can`, `Can`,
  `RequirePermission`), `context` (current org/project/pipeline + navigation), `di`, `grpc`,
  `query` (the query client and the Svelte bindings), `routing` (`ScyllaModule`, the route
  compilation, the router — written in the project, no router library).
- `features/` — `agents`, `apps`, `dashboard`, `jobs`, `login`, `marketplace`, `membership`,
  `organization`, `pipeline`, `project`, `roles`, `secret`, `triggers`, `user`.
- `layout/` — App shell (Layout, AppSidebar, ScyllaBreadcrumbs, context-selector).
- `shared/` — Reusable components/state helpers/stores/utils. **No business logic, no feature imports.**

### The `ScyllaModule` contract

Each module declares itself in `<feature>.module.ts` at its root, and the router, sidebar and DI
container are all *derived* from that one declaration — there is no second list to keep in sync:

```typescript
export const SecretModule = {
  id: 'secret',
  domain: { secretRepository },              // the DI surface
  routes: {
    project: [{                               // the mount: the shell grafts the route there
      path: 'secrets',
      permission: Permission.LIST_SECRETS,    // read by the route guard *and* the sidebar
      breadcrumb: () => ({ label: msg`Secrets` }),
      page: () => import('./presentation/ui/Secret.page.svelte'),
      // nav: { section, title, icon, order } — a sidebar link, organization routes only
      // children: [...]                       — routes below this path
    }],
  },
} satisfies ScyllaModule;
```

Rules that matter:
- `page` is a dynamic import: that is what keeps pages out of the initial chunk. Keep it.
- `permission` is declared **once** and drives both the route guard and the sidebar link. It
  guards that page only: a child route declares its own.
- Breadcrumb/nav labels are `` msg`…` `` descriptors, so the module file stays a `.ts`.
- Route parameters arrive as props of the page: `let { projectId }: { projectId?: string } = $props();`.
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

In query and mutation options, call `.unwrap()` inside `mutationFn`/`queryFn` so TanStack Query handles the error. `ScyllaError` extends `Error` with gRPC code extraction. Utilities live in `src/modules/shared/utils/`.

---

## Shared Patterns (reuse these — don't reinvent)

- **Selection**: `createSelection(key)` over the single `selectionStore`, keyed by feature. Used by
  `DataTable` + `FeatureHeader`. No per-feature selection store.
- **List headers**: `FeatureHeader` (count, clear/delete selection, new button).
- **Forms**: declarative `ScyllaForm` from `FormItem[]`; `FormDialog` wraps it in a dialog;
  `createFormState(() => items)` manages values/changes/reset/validation. Both are generic over
  the item ids — type the items `readonly FormItem<'a' | 'b'>[]` and `onSubmit` hands back a typed
  `FormValues` record, so never search the values by id.
- **Pagination**: `createPagination(options)` (local page state merged with server
  `totalCount`/`totalPages`).
- **Navigation**: `scyllaNavigate` and `navigateTo` from `@platform/context`. Never import the
  router.
- **Global state**: only `contextStore` (current org/project) and `selectionStore` are app-wide,
  plus `permissionsStore` in `@platform/authz`. Everything else = TanStack Query (server) or local
  `$state`.
- **Batch loads**: use `createQueries` to avoid N+1 (see jobs-per-pipeline).

### Query keys

Always use **factory functions** so queries and invalidations stay in sync:

```typescript
export const JOBS_QUERY_KEY = (pipelineId: string) =>
  ['jobs', 'pipeline', pipelineId, MAX_JOBS_PER_PIPELINE] as const;
// invalidate: queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(id), exact: true });
```

---

## Svelte — Best Practices

Svelte 5, **runes only** (`svelte.config.js` sets `runes: true`). No SvelteKit: the build is a
static SPA that the Rust binary serves. See `refacto_svelte.md` for the reasons.

### Where the logic goes

| Need | Solution | Where |
|------|----------|-------|
| Simple UI state (a toggle, a dialog, tabs) | `$state` | the `<script>` of the `.svelte` file |
| Page orchestration (queries + filters + pagination) | a ViewModel | `presentation/<view>.state.svelte.ts` |
| Complex UI logic without network (a wizard, a matrix) | a UI-only ViewModel | `presentation/<view>.state.svelte.ts` |
| A pure algorithm (a chart scale, a graph layout) | a pure function | `presentation/*.calculator.ts` |
| DOM measurement, native events, a third-party widget | a Svelte action | `presentation/*.actions.ts` |

**Never put non-trivial logic in a `.svelte` file.** A ViewModel is named after the view it
controls (`user-list.state.svelte.ts`), never after a global concept. A ViewModel is testable in
plain TypeScript, without a component.

### `$effect` is a last resort, not the default tool

An effect exists to synchronize with a system **outside** Svelte (the router, a timer, a
subscription, a CodeMirror or `@xyflow` instance). For everything else there is a better tool:

| Need | ❌ Not an effect | ✅ Instead |
|------|-----------------|-----------|
| Value derived from props/state | `$effect` + assignment | `$derived` / `$derived.by` |
| Server data | `$effect` + gRPC call | `createQuery` on a `*.queries.ts` factory |
| Respond to a user action | effect watching state | The event handler |
| Reset a form when a dialog opens | effect on `open` | `{#key open}` around a child component |
| Refetch after a mutation | effect | `invalidateQueries` in `onSuccess` |
| Touch the DOM | `bind:this` + effect | a Svelte action (`use:action`) — the only sanctioned way |

- **No mirror state.** What TanStack Query already holds (`data`, `isPending`, `error`) is never
  copied into `$state` or a store. Same for form state owned by `createFormState`, and for the
  current org/project owned by `contextStore`.
- **A list that arrives later is passed as a getter**, not as an array: `createFeatureSelection`
  takes `() => string[]`. A value read once freezes the helper on the first, usually empty,
  render.
- **Read a store from rune code with `toRune(store)`** (`shared/presentation/stores`). It
  subscribes only while something reads it.
- When an effect must read state that must not re-run it, read that state with `untrack`.

### Data access

- **A component never reaches the domain directly.** Only a `*.queries.ts` or a
  `*.state.svelte.ts` calls `getModuleDomain`.
- One remote operation = one entry in a `*.queries.ts` factory, built on a query-key factory.
- **Import `createQuery` / `createMutation` from `@platform/query`**, never from
  `@tanstack/svelte-query` (`no-restricted-imports`).
- Lists go through `DataTable` + `createPagination()`; don't render thousands of unpaginated
  rows. Row keys must be stable business ids, never array indices.

### Component splitting

- **One component = one responsibility.** Past ~150 lines, or as soon as you have to scroll to
  follow the markup, split it.
- Logic moves into a ViewModel, an action or a calculator; the component keeps rendering only.
- Self-contained or repeated markup becomes a named component, or a `{#snippet}` when it is
  local to one file. A snippet can call itself — use that for recursion, not a component that
  imports itself (`no-circular`).
- A component used by ≥ 2 features moves up to `shared/presentation/ui/` (and gets exported from
  the relevant barrel `index.ts`).
- `tsc` sees only the default export of a `.svelte` file. Anything a `.ts` file imports — a
  `cva` config, a type — lives in a `.ts` file beside the component.

### Minimalism (applies inside the layers, not against them)

The 4-layer structure is mandatory; everything else must earn its place. Before adding a file, an abstraction, a store, or a dependency, ask: **what breaks if I don't?** If the answer is "nothing yet", don't.

- No speculative abstraction. Factor at the 2nd or 3rd real usage, never "just in case". A helper used once stays inline.
- Derive rather than store — an extra piece of state is an extra thing to keep in sync.
- Dead code is deleted, not commented out "for later" — git keeps the history.
- A new dependency must justify itself; 20 lines of local code is often the better trade.

---

## Naming Conventions (enforced)

**Files/folders: kebab-case. Svelte component files: PascalCase.**

| Thing | Pattern | Example |
|-------|---------|---------|
| Page component file | `*.page.svelte` | `UserAdmin.page.svelte` |
| Svelte component | `PascalCase.svelte` | `SecretList.svelte` |
| ViewModel (runes) | `<view>.state.svelte.ts` → `create{View}State` or a `{View}State` class | `roles-page.state.svelte.ts` |
| Rune helper (no view) | `*.svelte.ts` | `organization-sync.svelte.ts` |
| Query / mutation options | `<feature>.queries.ts` → `{feature}Queries`, `{feature}Mutations` | `user.queries.ts` / `userQueries` |
| Messages (i18n) | `*.messages.ts` → `{name}Messages` | `secret.messages.ts` / `secretMessages` |
| Svelte action | `*.actions.ts`, named by a verb | `code-mirror.actions.ts` / `renderCodeMirror` |
| Pure algorithm | `*.calculator.ts` | `outcomes-chart.calculator.ts` |
| Test fixture (a component a test renders) | `*.fixture.svelte` | `DataTable.fixture.svelte` |
| Use case (only when it orchestrates) | `*.use-case.ts` → `{Verb}{Entity}UseCase` | `update-role.use-case.ts` / `UpdateRoleUseCase` |
| Repository interface | `*.repository.ts` (domain) → `{Entity}Repository` | `user.repository.ts` / `UserRepository` |
| Repository impl | `default-*.repository.ts` (infra) → `Default{Entity}Repository` | `default-user.repository.ts` / `DefaultUserRepository` |
| Data source iface | `*.data-source.ts` | `user-remote.data-source.ts` |
| Data source impl | `*.data-source.impl.ts` | `user-remote.data-source.impl.ts` |
| Mapper | `grpc-*.mapper.ts` → `Grpc{Entity}Mapper` | `grpc-user.mapper.ts` |
| Domain entity | `*.entity.ts` → `{Name}Entity` | `secret.entity.ts` / `SecretEntity` |
| Struct (value object / enum / DTO / wrapper) | `*.struct.ts` (plain name, no suffix) | `permission.struct.ts` / `Permission`, `pagination.struct.ts` / `PaginationInfo` |
| Store (framework-free, `createStore`) | `*.store.ts` → `{name}Store` | `context.store.ts` / `contextStore` |
| Module declaration (at module root) | `*.module.ts` → `{Feature}Module` | `user.module.ts` / `UserModule` |
| Module public API | `index.ts` | `features/membership/index.ts` |
| Guard / Wrapper | `*.guard.svelte` / `*.wrapper.svelte` | `Auth.guard.svelte` |
| Router | `*.router.ts` | `core.router.ts` |

Code identifiers: Interfaces/Types/Classes/Components/Enums **PascalCase** (no `I` prefix); factories of rune state **camelCase `create*`**; true constants **UPPER_SNAKE_CASE** (`DEFAULT_PAGE_SIZE`); query-key factories **UPPER_SNAKE_CASE**; props type = `Props` inside the component. There are no `use*` hooks.

### `interface` vs `type`

- **`interface`** for object shapes that are extended or implemented: component props (`interface Props`), repository contracts, data source contracts. This is the codebase majority — follow it.
- **`type`** for everything a shape can't express: unions, literal unions, mapped/utility types, function types, aliases (`type PermissionScope = 'org' | 'project'`).
- Type-only imports must use `import type` (`@typescript-eslint/consistent-type-imports` is an error, and `verbatimModuleSyntax` requires it).
- Colocate a type with the file that uses it; a dedicated file is only justified for shared or bulky types — and in `domain/` it must land in `entities/` or `structs/` per the rules above.

> Note: docs/naming-conventions.md shows `{Entity}RepositoryImpl` for the impl, but the actual codebase uses `Default{Entity}Repository` in `default-{entity}.repository.ts`. **Follow the codebase convention.**

---

## Testing (Vitest)

Tests live **next to the code they cover** — `grant-creator.state.svelte.ts` →
`grant-creator.state.svelte.test.ts`, `GrantCreator.svelte` → `GrantCreator.test.ts`. There is no
`__tests__/` mirror tree.

The harness is five files in `src/test/`, and it is the only shared test code:

| | |
|---|---|
| `src/test/setup.ts` | Runs before every file. Activates an empty `en` catalog, and stubs the browser APIs jsdom lacks (`ResizeObserver`, `Element.animate`, pointer capture, `scrollIntoView`, `scrollTo`, `matchMedia`). **Never re-stub these per file.** |
| `src/test/render.svelte.ts` | `render`, `withRegistry` (stub DI registry), `withQueryClient` (fresh cache, `retry: false`), `focusSettled`, `textSnippet`, `findFloating` / `findTooltip`. |
| `src/test/navigator.ts` | `installTestNavigator` — a fake navigator, for a test that navigates. |
| `src/test/queries.ts` | `runQueryFn`, `runMutationFn`, `runOnSuccess`, `stubQuery`. |
| `src/test/i18n.ts` | `withLocale(locale, messages)` — for the handful of tests that assert on a real translation. |

### The rules that bite here

- **A component is rendered with `render` from `@testing-library/svelte`.** There are no
  providers: install what the component reads (`withRegistry`, `withQueryClient`,
  `installTestNavigator`) and restore it in `afterEach` — it is module state.
- **A ViewModel is tested without a component**, in `$effect.root` + `flushSync`. A
  `*.queries.ts` factory is tested with `runQueryFn` / `runMutationFn`.
- A component with parts or a generic type is driven from a `*.fixture.svelte`.
- **A pure test opts out of jsdom** with `// @vitest-environment node` on the first line.
  Mappers, `domain/`, calculators — anything that never touches the DOM. A test that imports the
  router (`@platform/routing`) needs jsdom.
- **Query by role and accessible name**, not by CSS class. An icon-only control that can't be
  found by name is a missing `sr-only` label in the *component*, not a reason to reach for
  `querySelector`. App-owned attributes (`data-slot`, `data-variant`) are fair game; a third
  party's class names are not.
- **Inject a fake repository through the DI registry** (`withRegistry`). Don't mock the query
  under test; mock the boundary beneath it.
- **Don't mock `can`/`permissionsStore` away.** Drive the real store with
  `permissionsStore.setState(...)`, so the authorization chain is actually exercised.
- **Name a test after the rule it pins**, not the action it performs: "still redirects for an
  empty-string token" beats "test token". The suite output is the spec.
- **No snapshots.** None exist; keep it that way.

### Permission conformance — the one test that enumerates

`src/modules/core/di/module-permissions.test.ts` holds the whole app to one rule, derived from
the compiled routes (`compileRoutes(appRoutes)`) rather than from a hand-written list:

1. every page behind `AuthGuard` declares its own `permission` — there is no inheritance from a
   parent route.

A sidebar entry needs no rule: `nav` is part of its route and takes the route's permission.

**A new page is checked the day its module joins the registry**, with no test to remember to
write. That is the point: a per-component test pins a gate that exists, this one fails for a
gate that doesn't. If a page genuinely needs no permission, add it to `UNGATED_PAGES` **with the
reason** — a ratchet, like the coverage thresholds, and a stale entry fails the suite too.

`feature-permissions.test.ts` applies the same idea one level down, by reading source because
the gating of a *button* is declared nowhere a type can see it:

2. a feature that declares a mutation must mention a `Permission` somewhere under its
   `presentation/ui/`;
3. a query another feature imports through the barrel must check for itself — crossing a
   barrel means running outside the owner's route guard, on the *consumer's* permission.
   `jobsByPipelinesQueries` is the model: `enabled: ready && can(...)`.

All three rules answer **completeness, not correctness**: they cannot tell you the permission on
a button is the wrong one. That stays the job of the per-component tests. `UNGATED_FEATURES` and
`UNCHECKED_SHARED_HOOKS` are ratchets seeded with today's state; `SEEDED DEBT` and `TRIAGE`
entries are open questions, not decisions.

### Coverage

`pnpm coverage` enforces the thresholds in `vite.config.ts`, and CI runs it in place of
`pnpm test`. They are a **ratchet**: raise them when a batch of tests lands, never lower them to
turn a red run green. Generated proto code, compiled catalogs, vendored `shadcn/`, barrels and
`*.module.ts` are excluded — covering a re-export measures nothing.

---

## i18n (Lingui)

- **`lingui extract` does not read `.svelte` files.** Declare every message of a component with
  `` msg`…` `` in a `*.messages.ts` beside it, and render it with `t()` from
  `shared/presentation/utils/i18n-svelte.svelte.ts` (reactive to a locale switch). A message
  written in a `.svelte` file disappears from the catalogs, and no gate fails.
- A message with a placeholder is a function: `` newEntity: (label: string) => msg`New ${label}` ``.
  **When you port a message, keep its placeholder names**: they are part of the msgid.
- The `msg`, `t` and `plural` macros of `@lingui/core/macro` compile in `.ts` files only, through
  Babel (`linguiMacros` in `vite.config.ts`).
- Catalogs live in `locales/{en,fr}/messages.po` per feature (+ global). Run `pnpm extract` after adding strings, `pnpm compile` to build catalogs. Don't edit `messages.ts` by hand.
- **Catalogs are per-module, but the runtime merges them into one flat map** keyed by a hash of
  message + context, last one loaded winning. So two modules translating the same source string
  differently overwrite each other *app-wide* — and an untranslated twin renders as English while
  every catalog still looks complete. `pnpm i18n:collisions` is the CI gate; keep it at zero.
- When the same English word needs two French forms, **give one a `context`** rather than living
  with the collision — it is part of the hash, so it separates them cleanly:
  `` msg({ context: 'date-prefix', message: 'Created' }) `` → "Créé le" vs. the bare column header "Créé";
  `msg({ context: 'feminine', message: 'Unknown' })` → "Inconnue" vs. "Inconnu".
  When the two uses genuinely mean the same thing, unify the wording instead.
- French copy uses **straight apostrophes** (`'`), never `’` — the two are different characters and
  produce two different messages for the same string.
- `i18n:collisions` compares catalogs; it cannot tell you a French plural or interpolation
  actually renders. That is what the `*.fr.test.ts` files do, via `withLocale` — a handful of
  them, on the messages with plural arms and placeholders. Add one when you add a message whose
  French form is structurally different from the English (`_0` arms, gendered agreement).

---

## Conventions & tooling

- Path alias `@/` → `src/` (e.g. `@/modules/features/user/...`).
- Prettier: semicolons, single quotes (incl. JSX), 2-space tabs, trailing commas (all), printWidth 100, `arrowParens: avoid`. Match this style; don't reformat unrelated code.
- Routing: written in the project, in `@platform/routing` (no router library). The mounts are
  in `core/presentation/ui/router/core.router.ts`; the routes of the `app` mount and below are
  wrapped by `AuthGuard` + `Layout` (`AppShell.svelte`).
- Backend comms: gRPC-Web via protobuf-ts through `CoreGrpcTransport`.
- Comments: **none in the code, except on public/exported items.** When the reason behind the
  code needs more room, write it in the module's `AGENTS.md`, not inline. Team-visible text
  (PR bodies, issues, `AGENTS.md`, comments on public items) is written in ASD-STE100.
- Lint rules worth knowing (see `eslint.config.js`): `no-floating-promises` and `no-misused-promises` are errors — never fire-and-forget a promise; unused bindings must be prefixed `_` to be tolerated. The `no-unsafe-*` rules are off only because of the generated proto layer — that is not a licence to spread `any`. `no-restricted-imports` forbids `@tanstack/svelte-query` (use `@platform/query`).

### Stack
Svelte 5 (runes) · TypeScript 5.8 · TanStack Query 5 (`@tanstack/svelte-query`) · TanStack Table 9 · Lingui 5 · gRPC-Web (protobuf-ts) · shadcn-svelte + bits-ui · lucide (`@lucide/svelte`) · svelte-sonner · `@xyflow/svelte` · CodeMirror 6 · Tailwind CSS 4 · Vite 7 · Vitest + Testing Library.

---

## Adding a feature (checklist)

1. Create `src/modules/features/<feature>/` with `domain/ infrastructure/ locales/ presentation/`.
2. Domain: repository interface (+ its input types), `entities/*.entity.ts` and `structs/*.struct.ts`.
   **Add a use case only if it orchestrates** — see "Use cases are optional".
3. Infrastructure: data source (iface + `.impl`), `default-<feature>.repository.ts`, `grpc-<feature>.mapper.ts`.
4. `<feature>.module.ts` at the module root: `{ id, domain, routes? } satisfies ScyllaModule`,
   using `grpcTransport` from `@platform/grpc`. Each route loads its page with
   `page: () => import('./presentation/ui/X.page.svelte')`. Register the module in
   `core/di/registry.ts`.
5. `presentation/<feature>.queries.ts`: query-key factories, `queryOptions` / `mutationOptions`
   factories. The repository comes from
   `getModuleDomain<typeof XModule.domain>('<id>')` — in this file only.
6. Presentation: follow "Where the logic goes" — a `*.state.svelte.ts` ViewModel for a page that
   orchestrates, actions for DOM work, calculators for pure algorithms. Components import shared
   UI from `@shared/presentation/ui` and primitives from `@shadcn`. Read
   `src/modules/shared/AGENTS.md` first.
7. Every message in a `*.messages.ts` beside its component (`msg`), rendered with `t()`. Add
   `locales/{en,fr}/` and register the catalog in `lingui.config.js`.
8. `index.ts`: export only what other modules may use — never the `*.module.ts`, never a
   component (export a loader), and a page only when another module composes it behind its own
   route. Every feature has one, even when nothing consumes it yet.
9. Reuse before adding: check `shared/` and the Shared Patterns section first.
10. `AGENTS.md` + `README.md` at the module root, and a row in the root `README.md`'s module
    table. Follow the shape of a neighbouring module's pair: `AGENTS.md` = public API, data
    contract, file map, routes/nav, the rules that bite there; `README.md` = what it is for and
    why it is built that way.
11. Tests next to the code they cover (`*.test.ts`) — see "Testing" above. Run `pnpm coverage`:
    the thresholds are the gate that sees a module arrive without tests.
12. `pnpm typecheck && pnpm test && pnpm lint && pnpm depcruise && pnpm depcruise:cycles &&
    pnpm i18n:collisions` all clean.

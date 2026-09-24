# `platform/di` — agent guide

The dependency-injection mechanism. The *wiring* lives in `core/di/registry.ts`, not here.

**Layer** `platform/` · alias `@platform/di`

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- Consumers import `@platform/di` — the barrel, never a deep path.

## Public API — `index.ts`

```typescript
type DomainRegistry
getModuleDomain            // one module's domain, read from the registry
setDependencyRegistry      // installed by core; replaced by a test
```

`core/di/registry.ts` calls `setDependencyRegistry(dependencies)` at import time, so the
registry is installed before the first render. There is no provider and no context.

**A test that reaches the domain installs a stub registry**, with `withRegistry` from
`src/test/render.svelte.ts`, and restores it in `afterEach`. The registry is module state, so it
leaks between tests otherwise.

## Layout

```
index.ts                             public API
dependencies.registry.ts             DomainRegistry, the registry, getModuleDomain
```

## `DomainRegistry` is untyped per module — on purpose

```typescript
export type DomainRegistry = Readonly<Record<string, object>>;
```

If this type named each module's domain, every feature reading a dependency would transitively
depend on **every other feature**. The registry is deliberately opaque; features pin the type on
their own side, in one place per feature:

```typescript
// features/organization/presentation/organization.queries.ts
const repository = () =>
  getModuleDomain<typeof OrganizationModule.domain>('organization').organizationRepository;
```

`typeof XModule.domain` keeps the cast honest — the type follows the module declaration.

**Only a `*.queries.ts` or a `*.state.svelte.ts` calls `getModuleDomain`.** A component never
reaches the domain.

## Failure modes

`getModuleDomain` throws, loudly, in two cases:

| Error | Cause |
|---|---|
| `No dependency registry set` | a test did not install a registry |
| `No module registered under id "x"` | the module isn't in `core/di/registry.ts`, or the string id is misspelt |

## Rules that bite here

- **Mechanism here, wiring in the app.** `core/di/registry.ts` imports every
  `<feature>.module.ts` and builds the map. This module must not know a single module id.
- The registry imports `<feature>.module.ts` **directly by path**, never `<feature>/index.ts` —
  the barrel re-exports UI, and importing it here would pull every page into the entry chunk.
  Enforced by `module-declaration-is-private`.
- A feature calls `getModuleDomain` only with its **own** id. Another module that reads the
  repository of a feature queries it behind the back of its `*.queries.ts`, and forks the query
  cache into two keys for one resource. Use the queries of that feature through its `index.ts`.

## Adding a feature to DI

1. `<feature>.module.ts` → `{ id: 'x', domain: { xRepository } } satisfies ScyllaModule`.
2. Register it in `core/di/registry.ts`.
3. In `presentation/x.queries.ts`: `getModuleDomain<typeof XModule.domain>('x').xRepository`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.

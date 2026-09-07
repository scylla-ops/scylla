# `platform/di` — agent guide

The dependency-injection mechanism. The *wiring* lives in `core/di/registry.ts`, not here.

**Layer** `platform/` · alias `@platform/di`

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- Consumers import `@platform/di` — the barrel, never a deep path.

## Public API — `index.ts`

```typescript
DependenciesContext, type DomainRegistry
DependenciesProvider
useModuleDomain
```

## Layout

```
index.ts                             public API
dependencies.context.ts              the React context + DomainRegistry type
Dependencies.provider.tsx            provider — mounted once by core
use-module-domain.ts                 the typed accessor
```

## `DomainRegistry` is untyped per module — on purpose

```typescript
export type DomainRegistry = Readonly<Record<string, object>>;
```

If this type named each module's domain, every feature reading a dependency would transitively
depend on **every other feature**. The registry is deliberately opaque; features pin the type on
their own side.

```typescript
// features/jobs/presentation/hooks/use-jobs-domain.ts   ← one per feature, private
export const useJobsDomain = () => useModuleDomain<typeof JobsModule.domain>('jobs');
```

That accessor is the single place per feature where the cast happens, and `typeof
XModule.domain` keeps it honest — the type follows the module declaration automatically.

**Never call `useModuleDomain` directly from a hook or a component.** Always go through the
feature's accessor.

## Failure modes

`useModuleDomain` throws, loudly, in two cases:

| Error | Cause |
|---|---|
| `must be used within a DependenciesProvider` | called outside the provider — usually a test without the wrapper |
| `No module registered under id "x"` | the module isn't in `core/di/registry.ts`, or the string id is misspelt |

The second is the common one when adding a feature: the accessor's id string must match
`Module.id` **and** the module must be listed in the registry.

## Rules that bite here

- **Mechanism here, wiring in the app.** `core/di/registry.ts` imports every
  `<feature>.module.ts` and builds the map. This module must not know a single module id.
- The registry imports `<feature>.module.ts` **directly by path**, never `<feature>/index.ts` —
  the barrel re-exports UI, and importing it here would pull every page into the entry chunk.
  Enforced by `module-declaration-is-private`.
- `use-<feature>-domain.ts` is private to its feature (`domain-accessor-is-private`, error).
  Another module calling it would query that repository behind its hooks' back and fork the
  query cache into two keys for one resource.
- One provider, mounted once by `core`. Do not nest a second one.

## Adding a feature to DI

1. `<feature>.module.ts` → `{ id: 'x', domain: { xRepository } } satisfies ScyllaModule`.
2. Register it in `core/di/registry.ts`.
3. `presentation/hooks/use-x-domain.ts` → `useModuleDomain<typeof XModule.domain>('x')`.
4. Hooks call `const { xRepository } = useXDomain();` — components never do.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.

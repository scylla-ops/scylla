# platform / di

> [Scylla frontend](../../../../README.md) › `platform/` › **di** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

Dependency injection: how a feature's hooks get hold of its repository without importing the
concrete implementation.

Three exports, imported as `@platform/di`: `DependenciesProvider`, `useModuleDomain`, and the
`DependenciesContext` / `DomainRegistry` behind them.

## Mechanism here, wiring in the app

This module provides the *plumbing* and knows nothing about what flows through it. It contains
no module ids, no repository types, no feature imports — it could be lifted into another
application unchanged.

The wiring is the composition root's job. `core/di/registry.ts` imports every
`<feature>.module.ts`, collects their `domain` objects into a map keyed by module id, and hands
it to `DependenciesProvider`. That separation is what lets a feature declare its dependencies
without importing the app that assembles them — which is what previously made every feature and
`core/` mutually dependent.

## The registry is deliberately opaque

```typescript
export type DomainRegistry = Readonly<Record<string, object>>;
```

It would be nicer, at first glance, to type this precisely — a record naming every module and
its domain. It would also be a disaster: any feature reading one dependency would transitively
depend on the type of *every other feature*, and the module graph would collapse into one node.

So the context stays opaque, and each feature pins the type on its own side:

```typescript
// features/jobs/presentation/hooks/use-jobs-domain.ts
export const useJobsDomain = () => useModuleDomain<typeof JobsModule.domain>('jobs');
```

There is exactly one such accessor per feature, and it is the single place where the cast
happens. Because the type parameter is `typeof JobsModule.domain`, it follows the module
declaration automatically — add a repository to the module and the accessor's type widens with
no second edit.

## Why the accessor is private

`use-<feature>-domain.ts` is never exported from a feature's `index.ts`, and dependency-cruiser
enforces it (`domain-accessor-is-private`).

The reason is cache integrity rather than purity. If module B called `useJobsDomain()`, it would
reach `jobs`'s repository directly, behind that module's hooks — and build its own query key for
a resource `jobs` already caches. One resource, two cache entries, and a mutation invalidating
one leaves the other stale on screen.

The supported path is to ask the owning feature for a hook and export it from its public API.
That is why `useOrganizationJobs` lives in `jobs` even though only the dashboard uses it.

## The chain, end to end

```
<feature>.module.ts        declares { id, domain: { xRepository } }
        │
core/di/registry.ts        imports every module declaration, builds the map
        │
DependenciesProvider       provides it, once, at the app root
        │
useModuleDomain<T>(id)     reads it back
        │
use-<feature>-domain.ts    pins T — the one cast per feature, private
        │
presentation hooks         const { xRepository } = useXDomain()
```

Components never appear in that chain. They consume hooks.

## When it throws

`useModuleDomain` fails loudly rather than returning `undefined`. Called outside the provider it
says so — usually a test missing the wrapper. Called with an unregistered id it names the id,
which is almost always a new feature that was given an accessor but never added to
`core/di/registry.ts`, or an id string that does not match `Module.id`.

## Related modules

- [core](../../core/README.md) — owns `di/registry.ts` and mounts the provider.
- [platform/routing](../routing/README.md) — the `ScyllaModule` contract whose `domain` field
  this consumes.
- [platform/grpc](../grpc/README.md) — the transport the concrete repositories are built on.

# `platform/query`

The cache every remote read in Scylla goes through.

## What it is for

TanStack Query holds the server state of the whole app: what the pipelines are,
which jobs ran, who the members of an organization are. There is exactly one
cache, and this module owns it.

Nothing else here. No hooks, no query keys, no wrappers — those belong to the
feature that owns the resource, next to the repository they call. This module
exists only so that "the cache" has an address that everyone can reach.

## Why it isn't declared in `App.tsx`

It used to be, and that was fine while there was one framework and one place
that mounted providers.

Two things changed it. First, the layer order: `core/` sits above `features/`,
so a client declared in the shell cannot be reached by a feature that needs it
outside of a React hook — during an invalidation from a plain function, for
instance. Second, and decisively, the migration to Svelte: a Svelte component
mounted inside a React page sees no React context at all, so the only way the
two can share a cache is for both to be handed the same module-level instance.

That second point is the whole reason islands work. A page still rendered in
React and a component already migrated to Svelte read the same cache entry, and
a mutation in one invalidates the other's query. Without it, the two frameworks
would quietly fetch the same resource twice and disagree about the result.

## The thing to watch

The React and Svelte bindings each pin `@tanstack/query-core` to an exact
version. If a bump leaves them on different ones, npm installs two copies, the
`QueryClient` built by one is a different object than the other expects, and the
cache forks — with nothing failing anywhere. `AGENTS.md` carries the check to run
after any bump.

## Error handling lives here too

Both caches report through one place, so that a failure produces exactly one
toast no matter which hook triggered it. The asymmetry between queries and
mutations is deliberate and explained in `AGENTS.md`: a query that cannot reach
the control plane signs the user out, a mutation does not.

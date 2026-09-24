# `platform/query`

The cache every remote read in Scylla goes through.

## What it is for

TanStack Query holds the server state of the whole app: what the pipelines are,
which jobs ran, who the members of an organization are. There is exactly one
cache, and this module owns it.

Nothing else here. No query keys, no feature queries — those belong to the
feature that owns the resource, next to the repository they call. This module
exists only so that "the cache" has an address that everyone can reach.

## Why it isn't declared in the shell

The layer order: `core/` sits above `features/`, so a client declared in the shell cannot be
reached by a feature that needs it outside a component — during an invalidation from a plain
function, for instance. So the client is a module-level instance here, and the Svelte bindings
(`createQuery`, `createMutation`) are re-exported already bound to it. No component needs a
provider to find the cache.

## The thing to watch

`@tanstack/svelte-query` pins `@tanstack/query-core` to an exact version, and the app also
depends on `@tanstack/query-core` directly. If a bump leaves them on different versions, two
copies are installed and the cache forks — with nothing failing anywhere.

## Error handling lives here too

Both caches report through one place, so that a failure produces exactly one
toast no matter which query or mutation triggered it. The asymmetry between queries and
mutations is deliberate and explained in `AGENTS.md`: a query that cannot reach
the control plane signs the user out, a mutation does not.

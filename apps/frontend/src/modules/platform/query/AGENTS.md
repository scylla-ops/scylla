# `platform/query` — AGENTS.md

The app's single TanStack Query cache, and the Svelte bindings that use it.

## Public API (`index.ts`)

| Export | What it is |
|---|---|
| `queryClient` | The one cache instance, built at module load |
| `getQueryClient`, `setQueryClient` | The active client; a test installs its own |
| `createQuery`, `createMutation`, `createQueries` | The `@tanstack/svelte-query` functions, bound to the active client |
| `queryOptions`, `mutationOptions` | Re-exported unchanged |
| `type CreateQueryResult`, `type CreateMutationResult` | Re-exported unchanged |

## File map

```
platform/query/
├── index.ts                public API
├── query-client.ts         the instance + the global error handlers
├── active-query-client.ts  getQueryClient / setQueryClient
└── svelte-query.ts         the bound createQuery / createMutation / createQueries
```

## Why it is a capability and not a line in the shell

**Features may not import the shell.** `core/` is above `features/` in the layer order, so a
client declared in the shell is unreachable from a feature — for example from a plain function
that invalidates a query.

## The rules that bite here

- **Import `createQuery` / `createMutation` from `@platform/query`, never from
  `@tanstack/svelte-query`.** The originals read the client from the Svelte context, and no
  component puts one there. `no-restricted-imports` enforces it.
- **`@tanstack/svelte-query` pins `@tanstack/query-core` to an exact version.** The direct
  dependency on `@tanstack/query-core` must name the same version. Two copies fork the cache
  silently.

## Error handling

`queryCache.onError` and `mutationCache.onError` are the app's single reporting point. **A query
or a mutation must not add its own `onError` toast** — the failure would be reported twice.

The two differ on purpose: a *query* that fails with a network error signs the user out (the UI
is served from the control plane's own origin, so "unreachable" and "no longer authenticated"
are indistinguishable from the browser), a *mutation* only toasts.

## Tests

`queryClient` is the production instance and is never used by the suite. A test installs its
own with `withQueryClient()` from `src/test/render.svelte.ts`, which sets `retry: false` —
without it a rejecting query is retried three times with backoff and the test times out instead
of reporting the error.

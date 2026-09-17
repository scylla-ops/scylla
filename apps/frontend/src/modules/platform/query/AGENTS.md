# `platform/query` — AGENTS.md

The app's single TanStack Query cache.

## Public API (`index.ts`)

| Export | Type | What it is |
|---|---|---|
| `queryClient` | `QueryClient` | The one cache instance, built at module load |

That is the whole surface. There are no hooks here: the bindings live with their
framework (`@tanstack/react-query`, `@tanstack/svelte-query`) and are handed
*this* client.

## File map

```
platform/query/
├── index.ts          public API
└── query-client.ts   the instance + the global error handlers
```

## Why it is a capability and not a line in `core/App.tsx`

Two reasons, both load-bearing:

1. **Features may not import the shell.** `core/` is above `features/` in the
   layer order, so a client declared in `App.tsx` is unreachable from a feature
   that needs it outside a React hook.
2. **Both frameworks must get the same instance.** During the React → Svelte
   migration, `QueryClientProvider` (React) and svelte-query's context are both
   given `queryClient`. A migrated module and a React one therefore share cache
   entries instead of each fetching the same resource under its own key.

## The rule that bites here

**`@tanstack/react-query` and `@tanstack/svelte-query` pin `@tanstack/query-core`
to an exact version each.** They must be bumped in lockstep to releases naming
the same one — today `5.103.1` for both.

Two copies of `query-core` fork the cache **silently**: nothing throws, no test
fails, and a mutation in a Svelte module simply stops invalidating the React
module's query. Check after any bump:

```sh
ls -d node_modules/.pnpm/@tanstack+query-core@*   # stale store entries are fine
node -e "const {createRequire}=require('module');
const rq=createRequire(require.resolve('@tanstack/react-query/package.json'));
const sq=createRequire(require.resolve('@tanstack/svelte-query/package.json'));
console.log(rq.resolve('@tanstack/query-core/package.json') === sq.resolve('@tanstack/query-core/package.json'));"
```

## Error handling

`queryCache.onError` and `mutationCache.onError` are the app's single reporting
point. **A hook must not add its own `onError` toast** — the failure would be
reported twice.

The two differ on purpose: a *query* that fails with a network error signs the
user out (the UI is served from the control plane's own origin, so "unreachable"
and "no longer authenticated" are indistinguishable from the browser), a
*mutation* only toasts.

## Tests

`queryClient` is the production instance and is never used by the suite. Tests
build their own through `createTestQueryClient()` in `src/test/render.tsx`,
which sets `retry: false` — without it a rejecting query is retried three times
with backoff and the test times out instead of reporting the error.

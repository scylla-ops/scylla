import {
  createQuery as createSvelteQuery,
  createMutation as createSvelteMutation,
  createQueries as createSvelteQueries,
} from '@tanstack/svelte-query';
import { getQueryClient } from './active-query-client.ts';

/**
 * TanStack's Svelte bindings, already holding the app's client.
 *
 * `createQuery(options)` on its own reads the client from Svelte context, put
 * there by a `QueryClientProvider` — and a Svelte island has no such ancestor:
 * it is mounted into a React tree, which is the whole point of Phase 0. Every
 * call would otherwise have to remember `createQuery(opts, () => queryClient)`,
 * and forgetting it throws at runtime rather than at build time.
 *
 * So the client is bound once, here. **Import `createQuery` from
 * `@platform/query`, never from `@tanstack/svelte-query`** — `eslint`'s
 * `no-restricted-imports` enforces that, because the two are indistinguishable
 * at the call site.
 *
 * The client is read per call rather than captured, so a test that installs its
 * own with `setQueryClient` is seen by components created afterwards.
 */

const client = () => getQueryClient();

// The casts keep TanStack's overloads — which carry the `initialData` and
// `select` narrowing — instead of collapsing them into one loose signature.
// Only the default client is added; an explicit one still wins.

export const createQuery = ((options: never, queryClient?: never) =>
  createSvelteQuery(options, queryClient ?? client)) as typeof createSvelteQuery;

export const createMutation = ((options: never, queryClient?: never) =>
  createSvelteMutation(options, queryClient ?? client)) as typeof createSvelteMutation;

export const createQueries = ((options: never, queryClient?: never) =>
  createSvelteQueries(options, queryClient ?? client)) as typeof createSvelteQueries;

export { queryOptions, mutationOptions } from '@tanstack/svelte-query';
export type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';

import {
  createQuery as createSvelteQuery,
  createMutation as createSvelteMutation,
  createQueries as createSvelteQueries,
} from '@tanstack/svelte-query';
import { getQueryClient } from './active-query-client.ts';

/**
 * TanStack's Svelte bindings, bound to the app's client (no component provides
 * one through the context). Always import them from `@platform/query`.
 */

const client = () => getQueryClient();

// The casts keep TanStack's overloads (`initialData`, `select` narrowing).

export const createQuery = ((options: never, queryClient?: never) =>
  createSvelteQuery(options, queryClient ?? client)) as typeof createSvelteQuery;

export const createMutation = ((options: never, queryClient?: never) =>
  createSvelteMutation(options, queryClient ?? client)) as typeof createSvelteMutation;

export const createQueries = ((options: never, queryClient?: never) =>
  createSvelteQueries(options, queryClient ?? client)) as typeof createSvelteQueries;

export { queryOptions, mutationOptions } from '@tanstack/svelte-query';
export type { CreateQueryResult, CreateMutationResult } from '@tanstack/svelte-query';

import type { QueryClient } from '@tanstack/query-core';
import { queryClient } from './query-client.ts';

/**
 * Which client the Svelte bindings use.
 *
 * The production answer is always `queryClient`; the override exists for tests,
 * which need `retry: false` and a cache that does not survive the file. Same
 * shape as `setDependencyRegistry` in `@platform/di`, and for the same reason:
 * Phase 0 replaced React's providers with module singletons, so substitution in
 * a test is a setter rather than a wrapper component.
 */
let active: QueryClient | null = null;

export const getQueryClient = (): QueryClient => active ?? queryClient;

/** Pass `null` to go back to the application's own client. */
export const setQueryClient = (client: QueryClient | null): void => {
  active = client;
};

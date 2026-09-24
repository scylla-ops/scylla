import type { QueryClient } from '@tanstack/query-core';
import { queryClient } from './query-client.ts';

/** Overridable so a test can use its own client (`retry: false`, a fresh cache). */
let active: QueryClient | null = null;

export const getQueryClient = (): QueryClient => active ?? queryClient;

export const setQueryClient = (client: QueryClient | null): void => {
  active = client;
};

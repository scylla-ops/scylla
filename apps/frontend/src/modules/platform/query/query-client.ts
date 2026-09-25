import { MutationCache, QueryCache, QueryClient } from '@tanstack/query-core';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

/**
 * A network failure also signs out: the UI is served by the control plane, so
 * "unreachable" and "no longer authenticated" look the same from here.
 */
const signOut = (): void => {
  localStorage.removeItem('token');
  window.location.href = '/login';
};

const reportError = (error: unknown, label: string, signOutOnNetworkError: boolean): void => {
  if (!(error instanceof ScyllaError)) {
    console.error(`${label} (Non-Scylla):`, error);
    return;
  }

  if (error.getCode() === 'UNAUTHENTICATED') {
    signOut();
    return;
  }

  error.log();

  if (signOutOnNetworkError && error.isNetworkError()) {
    signOut();
    return;
  }

  toast.error(error.userMessage());
};

/**
 * The app's one query cache.
 *
 * `@tanstack/query-core` must stay at the exact version that `@tanstack/svelte-query`
 * pins: two copies would silently split the cache.
 */
export const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: error => reportError(error, 'Query Error', true),
  }),
  // The only error toast for mutations: never add an `onError` toast in a mutation.
  mutationCache: new MutationCache({
    onError: error => reportError(error, 'Mutation Error', false),
  }),
  defaultOptions: {
    queries: {
      // `onError` above only runs once a query settles into its final error
      // state, after retries: an expired token cannot become valid between
      // attempts, so retrying it only delays the sign-out.
      retry: (failureCount, error) =>
        error instanceof ScyllaError && error.getCode() === 'UNAUTHENTICATED' ? false : failureCount < 3,
    },
  },
});

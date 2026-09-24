import { MutationCache, QueryCache, QueryClient } from '@tanstack/query-core';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

/**
 * Signing out on an error the session cannot recover from. A network failure is
 * treated the same way for queries: the control plane serves the UI from its own
 * origin, so "unreachable" and "no longer authenticated" are indistinguishable
 * from here, and the safe reading is the second one.
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
 * Built from `@tanstack/query-core` and held at module scope, so any module
 * reaches it without a provider.
 *
 * `@tanstack/svelte-query` pins `query-core` to an exact version. The direct
 * dependency on `@tanstack/query-core` must name the same version: two copies
 * would fork the cache silently.
 */
export const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: error => reportError(error, 'Query Error', true),
  }),
  // Global mutation error handler — individual hooks must NOT add their own
  // `onError` toast, or every failure is reported twice.
  mutationCache: new MutationCache({
    onError: error => reportError(error, 'Mutation Error', false),
  }),
});

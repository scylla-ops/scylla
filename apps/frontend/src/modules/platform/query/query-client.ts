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
 * Built from `@tanstack/query-core` rather than from either binding, and held at
 * module scope rather than in a provider: React reads it through
 * `QueryClientProvider` and Svelte through its own context, but both hand it
 * *this* instance. That is what lets a migrated module and a React one share a
 * cache entry instead of each fetching the same resource under its own key —
 * see `refacto_svelte.md` §3.
 *
 * `react-query` and `svelte-query` pin `query-core` to an exact version each, so
 * they must be bumped in lockstep to releases naming the same one. Two copies of
 * `query-core` would fork the cache silently, with nothing failing.
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

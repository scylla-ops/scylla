import { toast } from '@shared/presentation/utils/toast.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { navigateTo } from './navigator.ts';

export interface ResourceErrorOptions {
  /** The query's error, read through a getter so it can arrive later. */
  error: () => unknown;
  /** Where to send the user when the resource is gone (e.g. '..' for the list). */
  redirectTo: string;
  /** Toast shown on NOT_FOUND before redirecting. */
  notFoundMessage: string;
}

export interface ResourceError {
  /** True while the redirect is in flight — render nothing. */
  readonly redirecting: boolean;
  readonly scyllaError: ScyllaError | null;
}

/**
 * Maps gRPC error codes to UX on a detail page — the Svelte counterpart of
 * `use-resource-error.ts`.
 *
 * On NOT_FOUND (deleted, or never existed) it toasts and redirects to the list
 * rather than stranding the user on a broken page with a generic "failed to
 * fetch". Reuse it on every resource detail page instead of re-checking codes
 * inline.
 *
 * **It lives here rather than in `shared/presentation/state/` on purpose.** Its
 * React twin imported `useNavigate` straight from react-router; the agnostic
 * replacement is `navigateTo`, which belongs to this capability — and
 * `shared-is-generic` forbids `shared/` from importing `platform/`. Redirecting
 * off a dead resource is navigation policy, so this is where it lands.
 */
export const createResourceError = ({
  error,
  redirectTo,
  notFoundMessage,
}: ResourceErrorOptions): ResourceError => {
  const scyllaError = $derived(error() instanceof ScyllaError ? (error() as ScyllaError) : null);
  const notFound = $derived(!!scyllaError?.isNotFound());

  // The router is a system outside Svelte, which is what an effect is for. It
  // reads `notFound` and nothing else, so it fires once per transition into the
  // not-found state rather than on every error change.
  $effect(() => {
    if (!notFound) return;

    toast.error(notFoundMessage);
    navigateTo(redirectTo, { replace: true });
  });

  return {
    get redirecting() {
      return notFound;
    },
    get scyllaError() {
      return scyllaError;
    },
  };
};

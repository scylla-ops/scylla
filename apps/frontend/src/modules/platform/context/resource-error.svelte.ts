import { toast } from '@shared/presentation/utils/toast.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { navigateTo } from './navigator.ts';

export interface ResourceErrorOptions {
  /** A getter, so an error that arrives later is seen. */
  error: () => unknown;
  /** E.g. `'..'` for the list. */
  redirectTo: string;
  notFoundMessage: string;
}

export interface ResourceError {
  readonly redirecting: boolean;
  readonly scyllaError: ScyllaError | null;
}

/**
 * On NOT_FOUND, toasts and redirects instead of showing a broken detail page.
 * Use it on every resource detail page.
 */
export const createResourceError = ({
  error,
  redirectTo,
  notFoundMessage,
}: ResourceErrorOptions): ResourceError => {
  const scyllaError = $derived(error() instanceof ScyllaError ? (error() as ScyllaError) : null);
  const notFound = $derived(!!scyllaError?.isNotFound());

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

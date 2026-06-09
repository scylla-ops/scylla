import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { ScyllaError } from '@shared/utils/scylla-result.ts';

interface UseResourceErrorOptions {
  error: unknown;
  /** Where to send the user when the resource is gone (e.g. '..' for the list). */
  redirectTo: string;
  /** Toast shown on NOT_FOUND before redirecting. */
  notFoundMessage: string;
}

/**
 * Maps gRPC error codes to UX on a detail page. On NOT_FOUND (resource deleted
 * or never existed) it toasts and redirects to the list instead of stranding
 * the user on a broken page with a generic "failed to fetch". Returns
 * `redirecting` so the caller can render nothing while the redirect happens.
 *
 * Reuse this on every resource detail page rather than re-checking codes inline.
 */
export function useResourceError({ error, redirectTo, notFoundMessage }: UseResourceErrorOptions) {
  const navigate = useNavigate();
  const scyllaError = error instanceof ScyllaError ? error : null;
  const notFound = !!scyllaError?.isNotFound();

  useEffect(() => {
    if (notFound) {
      toast.error(notFoundMessage);
      void navigate(redirectTo, { replace: true });
    }
  }, [notFound, notFoundMessage, redirectTo, navigate]);

  return { redirecting: notFound, scyllaError };
}

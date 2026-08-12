import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';

const MY_PERMISSIONS_QUERY_KEY = 'authz-my-permissions';

/**
 * The signed-in user's own access, grouped by the scope each grant binds to.
 * A query rather than a mutation: it describes the current session and is read
 * on mount, unlike the admin lookup in `use-effective-permissions.ts`, which
 * runs on form submit for an arbitrary principal.
 *
 * Grants only change through an admin action elsewhere, so the result is cached
 * for the session; invalidate `MY_PERMISSIONS_QUERY_KEY` if the app ever needs
 * to reflect a mid-session change.
 */
export function useMyPermissions() {
  const { authz } = useDependencies();

  return useQuery({
    queryKey: [MY_PERMISSIONS_QUERY_KEY],
    queryFn: async () => (await authz.getMyPermissions.execute()).unwrap(),
  });
}

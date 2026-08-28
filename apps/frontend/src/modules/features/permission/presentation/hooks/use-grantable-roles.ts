import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { PermissionScope } from '@/modules/features/permission/domain/structs/permission.struct.ts';

export const GRANTABLE_ROLES_QUERY_KEY = (scope?: PermissionScope) =>
  ['permission-grantable-roles', scope ?? 'all'] as const;

/**
 * The roles that may be handed out at `scope`.
 *
 * Unlike the full catalog, this list needs no permission at all — it is a
 * compile-time constant on the backend, not tenant data — which makes it the
 * only role list an organization or project administrator can read. It carries
 * the builtins only; custom roles are grantable but absent here, so a view that
 * can also read the catalog should merge the two (see `useAssignableRoles`).
 */
export const useGrantableRoles = (scope?: PermissionScope) => {
  const { permission } = useDependencies();

  const query = useQuery({
    queryKey: GRANTABLE_ROLES_QUERY_KEY(scope),
    queryFn: async () => (await permission.listGrantableRoles.execute(scope)).unwrap(),
  });

  return {
    grantableRoles: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
  };
};

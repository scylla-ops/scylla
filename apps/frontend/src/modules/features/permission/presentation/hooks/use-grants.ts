import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { CreateGrantInput } from '@/modules/features/permission/domain/usecases/create-grant.use-case.ts';
import type { RevokeAllAccessInput } from '@/modules/features/permission/domain/usecases/revoke-all-access.use-case.ts';
import { useRefreshMyPermissions } from '@/modules/features/permission/presentation/hooks/use-refresh-my-permissions.ts';

const GRANTS_QUERY_KEY = 'permission-grants';

/**
 * List every grant (system-admin view) plus create/revoke mutations.
 */
export function useGrants() {
  const { permission } = useDependencies();
  const queryClient = useQueryClient();
  const refreshMyPermissions = useRefreshMyPermissions();
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: [GRANTS_QUERY_KEY] });
    // A grant change may alter what the current user can see — reload the
    // stored effective permissions that drive nav/button gating.
    void refreshMyPermissions();
  };

  const grantsQuery = useQuery({
    queryKey: [GRANTS_QUERY_KEY],
    queryFn: async () => (await permission.listGrants.execute()).unwrap(),
  });

  const createGrant = useMutation({
    mutationFn: async (input: CreateGrantInput) =>
      (await permission.createGrant.execute(input)).unwrap(),
    onSuccess: invalidate,
  });

  const revokeGrant = useMutation({
    mutationFn: async (id: string) => (await permission.revokeGrant.execute(id)).unwrap(),
    onSuccess: invalidate,
  });

  /**
   * Clears a principal's grants at a scope and beneath it. Distinct from
   * `revokeGrant`, which drops one grant by id: this is the "remove them from
   * here entirely" operation, and the only one that leaves no inert
   * project-scoped grant behind.
   */
  const revokeAllAccess = useMutation({
    mutationFn: async (input: RevokeAllAccessInput) =>
      (await permission.revokeAllAccess.execute(input)).unwrap(),
    onSuccess: invalidate,
  });

  return {
    grants: grantsQuery.data ?? [],
    isLoading: grantsQuery.isLoading,
    isError: grantsQuery.isError,
    error: grantsQuery.error,
    createGrant,
    revokeGrant,
    revokeAllAccess,
  };
}

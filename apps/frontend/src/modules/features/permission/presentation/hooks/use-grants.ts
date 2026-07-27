import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { CreateGrantInput } from '@/modules/features/permission/domain/usecases/create-grant.use-case.ts';

const GRANTS_QUERY_KEY = 'permission-grants';

/**
 * List every grant (system-admin view) plus create/revoke mutations.
 */
export function useGrants() {
  const { permission } = useDependencies();
  const queryClient = useQueryClient();
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: [GRANTS_QUERY_KEY] });
    // A grant change may alter what the current user can see — refresh the
    // cached effective permissions that drive nav/button gating.
    void queryClient.invalidateQueries({ queryKey: ['permission-effective'] });
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

  return {
    grants: grantsQuery.data ?? [],
    isLoading: grantsQuery.isLoading,
    isError: grantsQuery.isError,
    error: grantsQuery.error,
    createGrant,
    revokeGrant,
  };
}

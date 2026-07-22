import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { CreateGrantInput } from '@/modules/features/role/domain/usecases/create-grant.use-case.ts';

const GRANTS_QUERY_KEY = 'authz-grants';

/**
 * List every grant (system-admin view) plus create/revoke mutations.
 */
export function useGrants() {
  const { authz } = useDependencies();
  const queryClient = useQueryClient();
  const invalidate = () => queryClient.invalidateQueries({ queryKey: [GRANTS_QUERY_KEY] });

  const grantsQuery = useQuery({
    queryKey: [GRANTS_QUERY_KEY],
    queryFn: async () => (await authz.listGrants.execute()).unwrap(),
  });

  const createGrant = useMutation({
    mutationFn: async (input: CreateGrantInput) =>
      (await authz.createGrant.execute(input)).unwrap(),
    onSuccess: invalidate,
  });

  const revokeGrant = useMutation({
    mutationFn: async (id: string) => (await authz.revokeGrant.execute(id)).unwrap(),
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

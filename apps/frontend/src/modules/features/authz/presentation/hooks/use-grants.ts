import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { CreateGrantInput } from '@/modules/features/authz/domain/repository/authz.repository.ts';

const GRANTS_QUERY_KEY = 'authz-grants';

/**
 * List every grant (system-admin view) plus create/revoke mutations. The role
 * picker is sourced from the full role catalog (see `useRoles`) so custom roles
 * are grantable, not just the static builtin set.
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
    grants: grantsQuery.data?.grants ?? [],
    isLoading: grantsQuery.isLoading,
    isError: grantsQuery.isError,
    error: grantsQuery.error,
    createGrant,
    revokeGrant,
  };
}

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { CreateRoleInput } from '@/modules/features/permission/domain/usecases/create-role.use-case.ts';
import type { UpdateRoleInput } from '@/modules/features/permission/domain/usecases/update-role.use-case.ts';

const ROLES_QUERY_KEY = 'authz-roles';

/** List the dynamic role catalog + create/update/delete mutations. */
export function useRoles() {
  const { authz } = useDependencies();
  const queryClient = useQueryClient();
  const invalidate = () => queryClient.invalidateQueries({ queryKey: [ROLES_QUERY_KEY] });

  const query = useQuery({
    queryKey: [ROLES_QUERY_KEY],
    queryFn: async () => (await authz.listRoles.execute()).unwrap(),
  });

  const createRole = useMutation({
    mutationFn: async (input: CreateRoleInput) => (await authz.createRole.execute(input)).unwrap(),
    onSuccess: invalidate,
  });

  const updateRole = useMutation({
    mutationFn: async (input: UpdateRoleInput) => (await authz.updateRole.execute(input)).unwrap(),
    onSuccess: invalidate,
  });

  const deleteRole = useMutation({
    mutationFn: async (id: string) => (await authz.deleteRole.execute(id)).unwrap(),
    onSuccess: invalidate,
  });

  return {
    roles: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    createRole,
    updateRole,
    deleteRole,
  };
}

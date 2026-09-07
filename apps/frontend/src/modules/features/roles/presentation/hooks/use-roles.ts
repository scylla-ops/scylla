import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useRolesDomain } from '@/modules/features/roles/presentation/hooks/use-roles-domain.ts';
import type { RoleCreationData } from '@/modules/features/roles/domain/entities/role.entity.ts';
import type { UpdateRoleInput } from '@/modules/features/roles/domain/use-cases/update-role.use-case.ts';

const ROLES_QUERY_KEY = 'permission-roles';

/**
 * List the dynamic role catalog + create/update/delete mutations.
 *
 * The backend gates the whole catalog behind `MANAGE_ROLES`, so a caller that
 * only administers one organization or project cannot read it. Such a caller
 * passes `enabled: false` and works from the grantable-role catalog instead
 * (see {@link useGrantableRoles}) rather than asking for a denial.
 */
export function useRoles(options: { enabled?: boolean } = {}) {
  const { permissionRepository, updateRole: updateRoleUseCase } = useRolesDomain();
  const queryClient = useQueryClient();
  const invalidate = () => queryClient.invalidateQueries({ queryKey: [ROLES_QUERY_KEY] });

  const query = useQuery({
    queryKey: [ROLES_QUERY_KEY],
    queryFn: async () => (await permissionRepository.listRoles()).unwrap(),
    enabled: options.enabled ?? true,
  });

  const createRole = useMutation({
    mutationFn: async (input: RoleCreationData) =>
      (await permissionRepository.createRole(input)).unwrap(),
    onSuccess: invalidate,
  });

  const updateRole = useMutation({
    mutationFn: async (input: UpdateRoleInput) =>
      (await updateRoleUseCase.execute(input)).unwrap(),
    onSuccess: invalidate,
  });

  const deleteRole = useMutation({
    mutationFn: async (id: string) => (await permissionRepository.deleteRole(id)).unwrap(),
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

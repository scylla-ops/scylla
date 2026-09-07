import { useMutation } from '@tanstack/react-query';
import { useRolesDomain } from '@/modules/features/roles/presentation/hooks/use-roles-domain.ts';
import type { PrincipalEntity } from '@platform/authz/domain/structs/permission.struct.ts';

/**
 * On-demand "what can this principal do" lookup. Modelled as a mutation since
 * it runs when the user submits the form, not on mount.
 */
export function useEffectivePermissions() {
  const { permissionRepository } = useRolesDomain();

  return useMutation({
    mutationFn: async (principal: PrincipalEntity) =>
      (await permissionRepository.getEffectivePermissions(principal)).unwrap(),
  });
}

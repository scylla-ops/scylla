import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { PrincipalEntity } from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * On-demand "what can this principal do" lookup. Modelled as a mutation since
 * it runs when the user submits the form, not on mount.
 */
export function useEffectivePermissions() {
  const { permission } = useDependencies();

  return useMutation({
    mutationFn: async (principal: PrincipalEntity) =>
      (await permission.getEffectivePermissions.execute(principal)).unwrap(),
  });
}

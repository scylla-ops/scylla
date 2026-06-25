import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { PrincipalKind } from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * On-demand "what can this principal do" lookup. Modelled as a mutation since
 * it runs when the user submits the form, not on mount.
 */
export function useEffectivePermissions() {
  const { authz } = useDependencies();

  return useMutation({
    mutationFn: async (vars: { principalKind: PrincipalKind; principalId: string }) =>
      (await authz.getEffectivePermissions.execute(vars.principalKind, vars.principalId)).unwrap(),
  });
}

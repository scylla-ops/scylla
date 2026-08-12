import { useCallback } from 'react';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { PrincipalKind } from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * Fetches the signed-in user's effective permissions and writes them into the
 * shared context store — the single backend call behind all `can()` checks.
 * Call it after login, on context change ({@link usePermissionSync}) or after
 * anything that may alter the user's own grants.
 */
export const useRefreshMyPermissions = () => {
  const { getEffectivePermissions } = useDependencies().permission;
  const setPermissions = useContextStore(state => state.setPermissions);

  return useCallback(async () => {
    const userId = localStorage.getItem('userId') ?? '';
    if (userId === '') {
      // Not signed in — settled with no permissions rather than stuck loading.
      setPermissions({ scopes: [] });
      return;
    }
    const result = await getEffectivePermissions.execute({
      kind: PrincipalKind.USER,
      id: userId,
    });
    result.fold({
      onSuccess: permissions => setPermissions(permissions),
      // Failed lookup → settled as "no permissions": gated UI explains the
      // denial and the backend stays the real enforcer.
      onError: () => setPermissions({ scopes: [] }),
    });
  }, [getEffectivePermissions, setPermissions]);
};

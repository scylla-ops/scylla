import { useCallback } from 'react';
import { useRolesDomain } from '@/modules/features/roles/presentation/hooks/use-roles-domain.ts';
import { usePermissionsStore } from '@platform/authz/presentation/stores/use-permissions.store.ts';

/**
 * Fetches the signed-in user's effective permissions and writes them into the
 * permissions store — the single backend call behind all `can()` checks.
 * Call it after login, on context change ({@link usePermissionSync}) or after
 * anything that may alter the user's own grants.
 */
export const useRefreshMyPermissions = () => {
  const { permissionRepository } = useRolesDomain();
  const setPermissions = usePermissionsStore(state => state.setPermissions);

  return useCallback(async () => {
    const userId = localStorage.getItem('userId') ?? '';
    if (userId === '') {
      // Not signed in — settled with no permissions rather than stuck loading.
      setPermissions({ scopes: [] });
      return;
    }
    const result = await permissionRepository.getMyPermissions();
    result.fold({
      onSuccess: permissions => setPermissions(permissions),
      // Failed lookup → settled as "no permissions": gated UI explains the
      // denial and the backend stays the real enforcer.
      onError: () => setPermissions({ scopes: [] }),
    });
  }, [permissionRepository, setPermissions]);
};

import { useCallback } from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import {
  canAccess,
  type PermissionTarget,
} from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { useMyPermissions } from '@/modules/features/permission/presentation/hooks/use-my-permissions.ts';

/**
 * App-wide authorization for the current user. Returns `can(permission, target?)`,
 * a synchronous check that defaults the target to the current org/project context
 * — so `can(Permission.CREATE_PIPELINE)` asks "in the org/project I'm looking at".
 *
 * While the effective permissions are still loading (or the lookup failed) `can`
 * is **permissive**, so a slow/broken lookup never hides the whole UI. The backend
 * still enforces access — this only shapes what we bother showing.
 */
export const useAuthorization = () => {
  const { effective, isLoading, isError } = useMyPermissions();
  const orgId = useContextStore(state => state.organization.id);
  const projectId = useContextStore(state => state.project.id);

  const can = useCallback(
    (permission: Permission, target?: PermissionTarget): boolean => {
      if (!effective) return true; // not yet known → don't gate
      return canAccess(effective, permission, {
        organizationId: target?.organizationId ?? orgId ?? undefined,
        projectId: target?.projectId ?? projectId ?? undefined,
      });
    },
    [effective, orgId, projectId],
  );

  return { can, isLoading, isError, ready: !!effective };
};

/** Convenience: the boolean result of a single {@link useAuthorization} check. */
export const useCan = (permission: Permission, target?: PermissionTarget): boolean =>
  useAuthorization().can(permission, target);

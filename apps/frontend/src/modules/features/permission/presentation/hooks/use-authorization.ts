import { useCallback } from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import {
  canAccess,
  type PermissionTarget,
} from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';

/**
 * App-wide authorization for the current user. Returns `can(permission, target?)`,
 * a synchronous check that defaults the target to the current org/project context
 * — so `can(Permission.CREATE_PIPELINE)` asks "in the org/project I'm looking at".
 *
 * Reads the effective permissions loaded into the shared context store by
 * {@link usePermissionSync} — no backend call per check. `can` **denies while
 * the permissions are unknown** (store not yet populated): gated UI must never
 * flash content the user may not hold. Consumers that want a loading state
 * instead of a denial read `ready`.
 */
export const useAuthorization = () => {
  const effective = useContextStore(state => state.permissions);
  const orgId = useContextStore(state => state.organization.id);
  const projectId = useContextStore(state => state.project.id);

  const can = useCallback(
    (permission: Permission, target?: PermissionTarget): boolean => {
      if (!effective) return false; // unknown → deny; the backend is the enforcer anyway
      return canAccess(effective, permission, {
        organizationId: target?.organizationId ?? orgId ?? undefined,
        projectId: target?.projectId ?? projectId ?? undefined,
      });
    },
    [effective, orgId, projectId],
  );

  return { can, ready: effective !== null };
};

/** Convenience: the boolean result of a single {@link useAuthorization} check. */
export const useCan = (permission: Permission, target?: PermissionTarget): boolean =>
  useAuthorization().can(permission, target);

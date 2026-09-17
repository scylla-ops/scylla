import { useContextStore } from '@platform/context';
import { usePermissionsStore } from '@platform/authz/presentation/stores/use-permissions.store.ts';
import type { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import {
  canAccess,
  type PermissionTarget,
} from '@platform/authz/domain/entities/effective-permissions.entity.ts';

/**
 * Authorization without React.
 *
 * Same answer `useCan` gives, read straight from the stores rather than through
 * a hook — Zustand's `getState()` works outside a component, and the stores are
 * the single source of truth either way. This is what a Svelte island, an event
 * handler or a route guard calls; a React component still uses `useCan`, which
 * re-renders when the permissions land.
 *
 * **Denies while the permissions are unknown.** Gated UI must never flash
 * content the user may not hold; callers wanting a loading state read
 * {@link authorizationReady} instead.
 */
export const can = (permission: Permission, target?: PermissionTarget): boolean => {
  const effective = usePermissionsStore.getState().permissions;
  if (!effective) return false; // unknown → deny; the backend is the enforcer anyway

  const { organization, project } = useContextStore.getState();

  return canAccess(effective, permission, {
    organizationId: target?.organizationId ?? organization.id ?? undefined,
    projectId: target?.projectId ?? project.id ?? undefined,
  });
};

/** Whether the permissions have been loaded at all — `can` denies until they are. */
export const authorizationReady = (): boolean =>
  usePermissionsStore.getState().permissions !== null;

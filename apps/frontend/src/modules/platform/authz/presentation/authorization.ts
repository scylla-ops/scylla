import { contextStore } from '@platform/context';
import { permissionsStore } from '@platform/authz/presentation/stores/permissions.store.ts';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import type { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import {
  canAccess,
  type PermissionTarget,
} from '@platform/authz/domain/entities/effective-permissions.entity.ts';

const readPermissions = toRune(permissionsStore);
const readContext = toRune(contextStore);

/** Reactive in a `$derived`. Denies while the permissions are not loaded yet. */
export const can = (permission: Permission, target?: PermissionTarget): boolean => {
  const effective = readPermissions().permissions;
  if (!effective) return false;

  const { organization, project } = readContext();

  return canAccess(effective, permission, {
    organizationId: target?.organizationId ?? organization.id ?? undefined,
    projectId: target?.projectId ?? project.id ?? undefined,
  });
};

export const authorizationReady = (): boolean => readPermissions().permissions !== null;

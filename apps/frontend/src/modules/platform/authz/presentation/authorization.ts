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

/**
 * Whether the current user holds `permission`, read from the stores.
 *
 * The stores are read through `toRune`, so `$derived(can(…))` in a component
 * updates when the permissions arrive or the active project changes. Outside a
 * reactive context it is a plain synchronous function.
 *
 * **Denies while the permissions are unknown.** Gated UI must never show
 * content that the user may not hold; callers that want a loading state read
 * {@link authorizationReady}.
 */
export const can = (permission: Permission, target?: PermissionTarget): boolean => {
  const effective = readPermissions().permissions;
  if (!effective) return false;

  const { organization, project } = readContext();

  return canAccess(effective, permission, {
    organizationId: target?.organizationId ?? organization.id ?? undefined,
    projectId: target?.projectId ?? project.id ?? undefined,
  });
};

/** Whether the permissions have been loaded at all — `can` denies until they are. */
export const authorizationReady = (): boolean => readPermissions().permissions !== null;

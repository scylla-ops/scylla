import { useContextStore } from '@platform/context';
import { usePermissionsStore } from '@platform/authz/presentation/stores/use-permissions.store.ts';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import type { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import {
  canAccess,
  type PermissionTarget,
} from '@platform/authz/domain/entities/effective-permissions.entity.ts';

/**
 * Authorization without React.
 *
 * Same answer `useCan` gives, read straight from the stores rather than through
 * a hook — the stores are the single source of truth either way. This is what a
 * Svelte component, an event handler or a route guard calls; a React component
 * still uses `useCan`, which re-renders when the permissions land.
 *
 * **It is reactive where that means something.** The stores are read through
 * `toRune`, so `$derived(can(…))` in a Svelte component recomputes when the
 * permissions arrive or the active project changes — without it, a page
 * rendered before `usePermissionSync` resolves would stay denied for good.
 * Outside a reactive context — a click handler, a guard, a plain test —
 * `toRune` falls straight through to `getState()` and nothing subscribes, so
 * this stays a synchronous function with no lifecycle.
 *
 * **Denies while the permissions are unknown.** Gated UI must never flash
 * content the user may not hold; callers wanting a loading state read
 * {@link authorizationReady} instead.
 */
const readPermissions = toRune(usePermissionsStore);
const readContext = toRune(useContextStore);

export const can = (permission: Permission, target?: PermissionTarget): boolean => {
  const effective = readPermissions().permissions;
  if (!effective) return false; // unknown → deny; the backend is the enforcer anyway

  const { organization, project } = readContext();

  return canAccess(effective, permission, {
    organizationId: target?.organizationId ?? organization.id ?? undefined,
    projectId: target?.projectId ?? project.id ?? undefined,
  });
};

/** Whether the permissions have been loaded at all — `can` denies until they are. */
export const authorizationReady = (): boolean => readPermissions().permissions !== null;

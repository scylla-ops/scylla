/**
 * Authorization primitives.
 *
 * Everything here is read-only and dependency-free: `can` answers from a
 * store, it never calls the backend. That is what lets authz sit below the
 * features — any feature may gate its UI without depending on the feature that
 * administers roles and grants.
 *
 * Loading the store is the other side of the coin and stays in
 * `features/roles` (`syncMyPermissions`), because it needs a repository call.
 */
export {
  Permission,
  PermissionScope,
  PrincipalKind,
  RoleKind,
  type AccessEntity,
  type AccessSpec,
  type PrincipalEntity,
} from './domain/structs/permission.struct.ts';
export {
  canAccess,
  type EffectivePermissionsEntity,
  type EffectiveScopeEntity,
  type PermissionTarget,
} from './domain/entities/effective-permissions.entity.ts';
export { authorizationReady, can } from './presentation/authorization.ts';
export { permissionsStore } from './presentation/stores/permissions.store.ts';
export { default as Can } from './presentation/ui/Can.svelte';
export { default as PermissionDenied } from './presentation/ui/PermissionDenied.svelte';
export { default as RequirePermission } from './presentation/ui/RequirePermission.svelte';

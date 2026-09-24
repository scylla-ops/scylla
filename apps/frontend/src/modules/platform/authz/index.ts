/**
 * Read-only authorization: `can` reads a store and never calls the backend.
 * `features/roles` loads the store (`syncMyPermissions`).
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
export { default as Can } from './presentation/ui/Can/Can.svelte';
export { default as PermissionDenied } from './presentation/ui/PermissionDenied/PermissionDenied.svelte';
export { default as RequirePermission } from './presentation/ui/RequirePermission/RequirePermission.svelte';

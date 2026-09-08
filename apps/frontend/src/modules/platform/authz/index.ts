/**
 * Authorization primitives.
 *
 * Everything here is read-only and dependency-free: `useCan` answers from a
 * store, it never calls the backend. That is what lets authz sit below the
 * features — any feature may gate its UI without depending on the feature that
 * administers roles and grants.
 *
 * Loading the store is the other side of the coin and stays in
 * `features/roles` (`usePermissionSync`), because it needs a repository call.
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
export { useAuthorization, useCan } from './presentation/hooks/use-authorization.ts';
export { usePermissionsStore } from './presentation/stores/use-permissions.store.ts';
export { Can } from './presentation/ui/Can.tsx';
export { PermissionButton } from './presentation/ui/PermissionButton.tsx';
export { PermissionDenied } from './presentation/ui/PermissionDenied.tsx';
export { RequirePermission } from './presentation/ui/RequirePermission.tsx';

/**
 * Access control administration: the role catalog, grants, and the permission
 * vocabulary behind them.
 *
 * The public API of the module. The authorization *primitives* (`Permission`,
 * `useCan`) are not here — they live in `@platform/authz`, below the features,
 * so that gating a button never means depending on this module.
 *
 * `usePermissionSync` is the loading half of that split: it needs a backend
 * call, so it stays here and the shell mounts it once.
 */
export type { RoleEntity, RoleCreationData } from './domain/entities/role.entity.ts';
export type { GrantEntity } from './domain/entities/grant.entity.ts';
export { roleConfers } from './domain/entities/role.entity.ts';
export { useRoles } from './presentation/hooks/use-roles.ts';
export { useGrants, useScopedGrants, GRANTS_QUERY_KEY } from './presentation/hooks/use-grants.ts';
export { useGrantableRoles } from './presentation/hooks/use-grantable-roles.ts';
export { usePermissionLabels } from './presentation/hooks/use-permission-labels.ts';
export { usePermissionSync } from './presentation/hooks/use-permission-sync.ts';
export { humanizeRoleId } from './presentation/utils/role-label.ts';

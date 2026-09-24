/**
 * Role catalog, grants and permission vocabulary. The authorization primitives
 * (`Permission`, `can`) are in `@platform/authz`. Exports no component.
 */
export type { RoleEntity, RoleCreationData } from './domain/entities/role.entity.ts';
export type { GrantEntity } from './domain/entities/grant.entity.ts';
export { roleConfers } from './domain/entities/role.entity.ts';
export { humanizeRoleId } from './presentation/utils/role-label.ts';
export { scopeLabelOf } from './presentation/utils/permission-mapping.ts';
export {
  roleQueries,
  roleMutations,
  grantMutations,
  refreshMyPermissions,
  syncMyPermissions,
  resetPermissionSync,
  ROLES_QUERY_KEY,
  GRANTS_QUERY_KEY,
  GRANTABLE_ROLES_QUERY_KEY,
} from './presentation/roles.queries.ts';

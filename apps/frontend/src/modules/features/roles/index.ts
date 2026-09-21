/**
 * Access control administration: the role catalog, grants, and the permission
 * vocabulary behind them.
 *
 * The public API of the module. The authorization *primitives* (`Permission`,
 * `can`, `Can`) are not here — they live in `@platform/authz`, below the
 * features, so that gating a button never means depending on this module.
 *
 * `syncMyPermissions` is the loading half of that split: it needs a backend
 * call, so it stays here and the shell calls it once per context change.
 *
 * Every export below is framework-free — options factories, pure functions and
 * types. That is deliberate and enforced by use: this module went Svelte in
 * Phase 4 while `layout` and `membership` read from it from both sides, and a
 * `queryOptions` object is the only thing both `useQuery` and `createQuery` can
 * run against the same cache entry. **No component is exported** — a `.svelte`
 * re-exported from a barrel cannot be tree-shaken away by Rollup.
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

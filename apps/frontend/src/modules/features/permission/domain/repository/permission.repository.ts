import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  Permission,
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/models/permission.model.ts';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/permission/domain/entities/grant.entity.ts';
import type { GrantableRoleEntity } from '@/modules/features/permission/domain/entities/grantable-role.entity.ts';
import type { EffectivePermissionsEntity } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import type { AuthzVocabularyEntity } from '@/modules/features/permission/domain/entities/authz-vocabulary.entity.ts';

/**
 * Domain-level input for creating a grant.
 * Grants associate a principal (user) with a role or a single permission
 * within a given scope.
 */
export interface CreateGrantInput {
  /** String user id (value of the protobuf UserId wrapper). */
  userId?: string;
  /** The role to grant. Ignored when `permission` is set. */
  role: string;
  scope: PermissionScope;
  scopeId: string;
  /** Set to grant a single permission instead of a role. */
  permission?: Permission;
}

/**
 * Boundary onto the backend authorization services (RoleService + GrantService).
 * Implemented by {@link DefaultPermissionRepository} in the infrastructure layer.
 */
export interface PermissionRepository {
  // ── Roles (dynamic catalog) ────────────────────────────────────────────────
  listRoles(): Promise<ScyllaResult<RoleEntity[]>>;
  getRole(id: string): Promise<ScyllaResult<RoleEntity>>;
  createRole(role: RoleEntity): Promise<ScyllaResult<RoleEntity>>;
  updateRole(role: RoleEntity): Promise<ScyllaResult<RoleEntity>>;
  deleteRole(id: string): Promise<ScyllaResult<boolean>>;

  // ── Introspection ──────────────────────────────────────────────────────────
  getEffectivePermissions(
    principalKind: PrincipalKind,
    principalId: string,
  ): Promise<ScyllaResult<EffectivePermissionsEntity>>;

  // ── Grants (principal holds role|permission within scope) ──────────────────
  listGrants(scope?: PermissionScope, scopeId?: string): Promise<ScyllaResult<GrantEntity[]>>;
  createGrant(input: CreateGrantInput): Promise<ScyllaResult<GrantEntity>>;
  revokeGrant(id: string): Promise<ScyllaResult<boolean>>;
  listGrantableRoles(scope?: PermissionScope): Promise<ScyllaResult<GrantableRoleEntity[]>>;

  // ── Permission vocabulary ──────────────────────────────────────────────────
  listAuthzVocabulary(): Promise<ScyllaResult<AuthzVocabularyEntity>>;
}

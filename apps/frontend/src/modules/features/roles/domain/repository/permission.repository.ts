import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  PermissionScope,
  PrincipalEntity,
} from '@platform/authz/domain/structs/permission.struct.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/roles/domain/entities/role.entity.ts';
import type {
  GrantEntity,
} from '@/modules/features/roles/domain/entities/grant.entity.ts';
import type { GrantableRoleEntity } from '@/modules/features/roles/domain/entities/grantable-role.entity.ts';
import type { EffectivePermissionsEntity } from '@platform/authz/domain/entities/effective-permissions.entity.ts';
import type { PermissionVocabularyEntity } from '@/modules/features/roles/domain/entities/permission-vocabulary.entity.ts';

/**
 * Domain-level input for creating a grant.
 * Grants associate a principal (a user or an app) with a role or a single
 * permission within a given scope.
 */
export interface CreateGrantInput {
  principal: PrincipalEntity;
  /** The role, or the single permission, to grant. */
  /** The role to grant within the scope. */
  roleId: string;
  scope: PermissionScope;
  scopeId: string;
}

/**
 * Domain-level input for clearing a principal's access at a scope.
 */
export interface RevokeAllAccessInput {
  principal: PrincipalEntity;
  scope: PermissionScope;
  /** The org/project id to clear; empty for SYSTEM scope. */
  scopeId: string;
}

/**
 * Boundary onto the backend authorization services (RoleService + GrantService).
 * Implemented by {@link DefaultPermissionRepository} in the infrastructure layer.
 */
export interface PermissionRepository {
  // ── Roles (dynamic catalog) ────────────────────────────────────────────────
  listRoles(): Promise<ScyllaResult<RoleEntity[]>>;
  getRole(id: string): Promise<ScyllaResult<RoleEntity>>;
  createRole(role: RoleCreationData): Promise<ScyllaResult<RoleEntity>>;
  updateRole(role: RoleEntity): Promise<ScyllaResult<RoleEntity>>;
  deleteRole(id: string): Promise<ScyllaResult<void>>;

  // ── Introspection ──────────────────────────────────────────────────────────
  getEffectivePermissions(
    principal: PrincipalEntity,
  ): Promise<ScyllaResult<EffectivePermissionsEntity>>;
  /** The signed-in caller's own access. Needs no permission, unlike the above. */
  getMyPermissions(): Promise<ScyllaResult<EffectivePermissionsEntity>>;

  // ── Grants (principal holds role|permission within scope) ──────────────────
  listGrants(scope?: PermissionScope, scopeId?: string): Promise<ScyllaResult<GrantEntity[]>>;
  createGrant(input: CreateGrantInput): Promise<ScyllaResult<GrantEntity>>;
  revokeGrant(id: string): Promise<ScyllaResult<void>>;
  /**
   * Clears every grant `principal` holds at a scope *and beneath it*, answering
   * how many were removed.
   *
   * This is what removing someone from an organization means: membership is not
   * stored anywhere, it is derived from the grants held on the organization and
   * on its projects. Revoking only the organization-scoped ones would leave the
   * project-scoped ones behind — inert (the backend confers nothing on a project
   * grant whose organization grant is gone) yet still enough to keep the person
   * listed as a member.
   */
  revokeAllAccess(input: RevokeAllAccessInput): Promise<ScyllaResult<number>>;
  listGrantableRoles(scope?: PermissionScope): Promise<ScyllaResult<GrantableRoleEntity[]>>;

  // ── Permission vocabulary ──────────────────────────────────────────────────
  listPermissionVocabulary(): Promise<ScyllaResult<PermissionVocabularyEntity>>;
}

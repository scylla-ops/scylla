import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PermissionScope, PrincipalEntity, } from '@platform/authz';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/roles/domain/entities/role.entity.ts';
import type {
  GrantEntity,
} from '@/modules/features/roles/domain/entities/grant.entity.ts';
import type { GrantableRoleEntity } from '@/modules/features/roles/domain/entities/grantable-role.entity.ts';
import type { EffectivePermissionsEntity } from '@platform/authz';
import type { PermissionVocabularyEntity } from '@/modules/features/roles/domain/entities/permission-vocabulary.entity.ts';

export interface CreateGrantInput {
  principal: PrincipalEntity;
  roleId: string;
  scope: PermissionScope;
  scopeId: string;
}

export interface RevokeAllAccessInput {
  principal: PrincipalEntity;
  scope: PermissionScope;
  scopeId: string;
}

export interface PermissionRepository {
  listRoles(): Promise<ScyllaResult<RoleEntity[]>>;
  getRole(id: string): Promise<ScyllaResult<RoleEntity>>;
  createRole(role: RoleCreationData): Promise<ScyllaResult<RoleEntity>>;
  updateRole(role: RoleEntity): Promise<ScyllaResult<RoleEntity>>;
  deleteRole(id: string): Promise<ScyllaResult<void>>;

  getEffectivePermissions(
    principal: PrincipalEntity,
  ): Promise<ScyllaResult<EffectivePermissionsEntity>>;
  /** Needs no permission, unlike the above. */
  getMyPermissions(): Promise<ScyllaResult<EffectivePermissionsEntity>>;

  listGrants(scope?: PermissionScope, scopeId?: string): Promise<ScyllaResult<GrantEntity[]>>;
  createGrant(input: CreateGrantInput): Promise<ScyllaResult<GrantEntity>>;
  revokeGrant(id: string): Promise<ScyllaResult<void>>;
  /**
   * Clears every grant of `principal` at a scope and beneath it, and returns how many.
   * Membership is derived from grants: the project grants must go too.
   */
  revokeAllAccess(input: RevokeAllAccessInput): Promise<ScyllaResult<number>>;
  listGrantableRoles(scope?: PermissionScope): Promise<ScyllaResult<GrantableRoleEntity[]>>;

  listPermissionVocabulary(): Promise<ScyllaResult<PermissionVocabularyEntity>>;
}

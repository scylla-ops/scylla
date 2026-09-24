import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  AuthzAction,
  PrincipalRef,
  ScopeKind,
  ScopeRef,
} from '@/generated/scylla/authz/v1/permission.ts';
import type {
  CreateGrantRequest,
  Grant,
  GrantableRole,
} from '@/generated/scylla/authz/v1/grant.ts';
import type {
  CreateRoleRequest,
  EffectiveScope,
  Role,
  UpdateRoleRequest,
} from '@/generated/scylla/authz/v1/role.ts';

export interface PermissionDataSource {
  listRoles(): Promise<ScyllaResult<Role[]>>;
  getRoleById(id: string): Promise<ScyllaResult<Role>>;
  createRole(request: CreateRoleRequest): Promise<ScyllaResult<Role>>;
  updateRole(request: UpdateRoleRequest): Promise<ScyllaResult<Role>>;
  deleteRole(id: string): Promise<ScyllaResult<void>>;

  getEffectivePermissions(principal: PrincipalRef): Promise<ScyllaResult<EffectiveScope[]>>;
  getMyPermissions(): Promise<ScyllaResult<EffectiveScope[]>>;

  listGrants(scope?: ScopeRef): Promise<ScyllaResult<Grant[]>>;
  createGrant(request: CreateGrantRequest): Promise<ScyllaResult<Grant>>;
  revokeGrant(id: string): Promise<ScyllaResult<void>>;
  /** At `scope` and beneath it. */
  revokeAllAccess(principal: PrincipalRef, scope: ScopeRef): Promise<ScyllaResult<number>>;
  listGrantableRoles(scopeKind?: ScopeKind): Promise<ScyllaResult<GrantableRole[]>>;

  listPermissionVocabulary(): Promise<ScyllaResult<AuthzAction[]>>;
}

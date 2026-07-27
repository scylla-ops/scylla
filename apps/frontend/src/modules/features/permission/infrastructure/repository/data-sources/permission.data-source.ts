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

/**
 * Infrastructure boundary: raw gRPC calls to the authz backend
 * (`scylla.authz.v1`: RoleService + GrantService + PolicyService).
 * Every RPC now answers with an `XxxResponse` wrapper; this layer unwraps it so
 * callers keep receiving plain entities. Mapping to domain types is performed
 * by {@link DefaultPermissionRepository}.
 */
export interface PermissionDataSource {
  // ── Roles ──────────────────────────────────────────────────────────────────
  listRoles(): Promise<ScyllaResult<Role[]>>;
  getRoleById(id: string): Promise<ScyllaResult<Role>>;
  createRole(request: CreateRoleRequest): Promise<ScyllaResult<Role>>;
  updateRole(request: UpdateRoleRequest): Promise<ScyllaResult<Role>>;
  deleteRole(id: string): Promise<ScyllaResult<void>>;

  // ── Introspection ──────────────────────────────────────────────────────────
  getEffectivePermissions(principal: PrincipalRef): Promise<ScyllaResult<EffectiveScope[]>>;

  // ── Grants ─────────────────────────────────────────────────────────────────
  listGrants(scope?: ScopeRef): Promise<ScyllaResult<Grant[]>>;
  createGrant(request: CreateGrantRequest): Promise<ScyllaResult<Grant>>;
  revokeGrant(id: string): Promise<ScyllaResult<void>>;
  listGrantableRoles(scopeKind?: ScopeKind): Promise<ScyllaResult<GrantableRole[]>>;

  // ── Vocabulary ─────────────────────────────────────────────────────────────
  listPermissionVocabulary(): Promise<ScyllaResult<AuthzAction[]>>;
}

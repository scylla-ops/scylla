import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  DeleteRoleResponse,
  GetEffectivePermissionsResponse,
  Grant,
  ListGrantableRolesResponse,
  ListGrantsResponse,
  ListRolesResponse,
  Permission,
  PrincipalKind,
  RevokeGrantResponse,
  Role,
  Scope,
} from '@/generated/permission.ts';

/** Create a dynamic role (a named, editable bundle of permissions at a scope). */
export interface CreateRoleInput {
  name: string;
  description: string;
  scope: Scope;
  fullControl: boolean;
  permissions: Permission[];
}

/** Replace a role's name/description/permission set wholesale (scope is immutable). */
export interface UpdateRoleInput {
  id: string;
  name: string;
  description: string;
  fullControl: boolean;
  permissions: Permission[];
}

/** Grant a role OR a single permission to a user within a scope. */
export interface CreateGrantInput {
  userId: string;
  /** Role id/key — used unless `permission` is set. */
  role: string;
  scope: Scope;
  /** Org/project id the grant binds to; ignored for SYSTEM. */
  scopeId: string;
  /** Set to grant a single permission instead of a role (additive). */
  permission?: Permission;
}

/**
 * Boundary onto the backend authorization services (RoleService + GrantService).
 * The infrastructure gRPC data source implements this directly.
 */
export interface AuthzRepository {
  // Roles (dynamic catalog).
  listRoles(): Promise<ScyllaResult<ListRolesResponse>>;
  createRole(input: CreateRoleInput): Promise<ScyllaResult<Role>>;
  updateRole(input: UpdateRoleInput): Promise<ScyllaResult<Role>>;
  deleteRole(id: string): Promise<ScyllaResult<DeleteRoleResponse>>;

  // Introspection: what a principal can do, grouped by scope.
  getEffectivePermissions(
    principalKind: PrincipalKind,
    principalId: string,
  ): Promise<ScyllaResult<GetEffectivePermissionsResponse>>;

  // Grants (principal holds role|permission within scope).
  listGrants(scope?: Scope, scopeId?: string): Promise<ScyllaResult<ListGrantsResponse>>;
  createGrant(input: CreateGrantInput): Promise<ScyllaResult<Grant>>;
  revokeGrant(id: string): Promise<ScyllaResult<RevokeGrantResponse>>;
  listGrantableRoles(scope?: Scope): Promise<ScyllaResult<ListGrantableRolesResponse>>;
}

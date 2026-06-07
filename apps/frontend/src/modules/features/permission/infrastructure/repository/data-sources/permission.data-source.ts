import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateGrantRequest,
  CreateRoleRequest,
  DeleteRoleResponse,
  GetEffectivePermissionsResponse,
  Grant,
  ListAuthzVocabularyResponse,
  ListGrantableRolesResponse,
  ListGrantsResponse,
  ListRolesResponse,
  PrincipalKind,
  RevokeGrantResponse,
  Role,
  Scope,
  UpdateRoleRequest,
} from '@/generated/permission.ts';

/**
 * Infrastructure boundary: raw gRPC calls to the permission backend.
 * Returns generated protobuf types directly — mapping to domain types is
 * performed by {@link DefaultPermissionRepository}.
 */
export interface PermissionDataSource {
  // ── Roles ──────────────────────────────────────────────────────────────────
  listRoles(): Promise<ScyllaResult<ListRolesResponse>>;
  getRoleById(id: string): Promise<ScyllaResult<Role>>;
  createRole(request: CreateRoleRequest): Promise<ScyllaResult<Role>>;
  updateRole(request: UpdateRoleRequest): Promise<ScyllaResult<Role>>;
  deleteRole(id: string): Promise<ScyllaResult<DeleteRoleResponse>>;

  // ── Introspection ──────────────────────────────────────────────────────────
  getEffectivePermissions(
    principalKind: PrincipalKind,
    principalId: string,
  ): Promise<ScyllaResult<GetEffectivePermissionsResponse>>;

  // ── Grants ─────────────────────────────────────────────────────────────────
  listGrants(scope?: Scope, scopeId?: string): Promise<ScyllaResult<ListGrantsResponse>>;
  createGrant(request: CreateGrantRequest): Promise<ScyllaResult<Grant>>;
  revokeGrant(id: string): Promise<ScyllaResult<RevokeGrantResponse>>;
  listGrantableRoles(scope?: Scope): Promise<ScyllaResult<ListGrantableRolesResponse>>;

  // ── Vocabulary ─────────────────────────────────────────────────────────────
  listAuthzVocabulary(): Promise<ScyllaResult<ListAuthzVocabularyResponse>>;
}
import { GrantServiceClient } from '@/generated/scylla/authz/v1/grant.client.ts';
import { RoleServiceClient } from '@/generated/scylla/authz/v1/role.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
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

import type { PermissionDataSource } from '@/modules/features/permission/infrastructure/repository/data-sources/permission.data-source.ts';

/**
 * Reads the single entity a response wrapper is supposed to carry. The field is
 * `optional` on the wire but the backend always fills it on success, so an
 * absent one is a protocol error, not an empty result.
 */
const required = <T>(value: T | undefined, what: string): T => {
  if (value === undefined) throw new Error(`The server response carried no ${what}.`);
  return value;
};

/**
 * gRPC-backed implementation of {@link PermissionDataSource}.
 * Unwraps the `XxxResponse` wrappers so the repository above only ever sees
 * `scylla.authz.v1` entities.
 */
export class GrpcPermissionRemoteDataSource implements PermissionDataSource {
  private readonly _roles: RoleServiceClient;
  private readonly _grants: GrantServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._roles = new RoleServiceClient(transport.getTransport());
    this._grants = new GrantServiceClient(transport.getTransport());
  }

  public listRoles(): Promise<ScyllaResult<Role[]>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.listRoles({})).response.roles,
      'Failed to list roles.',
    );
  }

  public createRole(input: CreateRoleRequest): Promise<ScyllaResult<Role>> {
    return ScyllaResult.tryAsync(
      async () => required((await this._roles.createRole(input)).response.role, 'role'),
      'Failed to create role.',
    );
  }

  public getRoleById(id: string): Promise<ScyllaResult<Role>> {
    return ScyllaResult.tryAsync(
      async () =>
        required((await this._roles.getRole({ roleId: { value: id } })).response.role, 'role'),
      'Failed to fetch role.',
    );
  }

  public updateRole(input: UpdateRoleRequest): Promise<ScyllaResult<Role>> {
    return ScyllaResult.tryAsync(
      async () => required((await this._roles.updateRole(input)).response.role, 'role'),
      'Failed to update role.',
    );
  }

  public deleteRole(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._roles.deleteRole({ roleId: { value: id } });
    }, 'Failed to delete role.');
  }

  public getEffectivePermissions(principal: PrincipalRef): Promise<ScyllaResult<EffectiveScope[]>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.getEffectivePermissions({ principal })).response.scopes,
      'Failed to fetch effective permissions.',
    );
  }

  public listGrants(scope?: ScopeRef): Promise<ScyllaResult<Grant[]>> {
    return ScyllaResult.tryAsync(
      async () => (await this._grants.listGrants({ scope })).response.grants,
      'Failed to list grants.',
    );
  }

  public createGrant(input: CreateGrantRequest): Promise<ScyllaResult<Grant>> {
    return ScyllaResult.tryAsync(
      async () => required((await this._grants.createGrant(input)).response.grant, 'grant'),
      'Failed to create grant.',
    );
  }

  public revokeGrant(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._grants.revokeGrant({ grantId: { value: id } });
    }, 'Failed to revoke grant.');
  }

  public listGrantableRoles(scopeKind?: ScopeKind): Promise<ScyllaResult<GrantableRole[]>> {
    return ScyllaResult.tryAsync(
      async () => (await this._grants.listGrantableRoles({ scopeKind })).response.roles,
      'Failed to list grantable roles.',
    );
  }

  public listAuthzVocabulary(): Promise<ScyllaResult<AuthzAction[]>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.listAuthzVocabulary({})).response.actions,
      'Failed to load authz vocabulary.',
    );
  }
}

import {
  GrantServiceClient,
  PolicyServiceClient,
  RoleServiceClient,
} from '@/generated/permission.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
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

import type { PermissionDataSource } from '@/modules/features/permission/infrastructure/repository/data-sources/permission.data-source.ts';

/** gRPC-backed implementation of {@link PermissionRepository} */
export class GrpcPermissionRemoteDataSource implements PermissionDataSource {
  private readonly _roles: RoleServiceClient;
  private readonly _grants: GrantServiceClient;
  private readonly _policies: PolicyServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._roles = new RoleServiceClient(transport.getTransport());
    this._grants = new GrantServiceClient(transport.getTransport());
    this._policies = new PolicyServiceClient(transport.getTransport());
  }

  public listRoles(): Promise<ScyllaResult<ListRolesResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.listRoles({})).response,
      'Failed to list roles.',
    );
  }

  public createRole(input: CreateRoleRequest): Promise<ScyllaResult<Role>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.createRole({ ...input })).response,
      'Failed to create role.',
    );
  }

  public getRoleById(id: string): Promise<ScyllaResult<Role>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.getRole({ id })).response,
      'Failed to fetch role.',
    );
  }

  public updateRole(input: UpdateRoleRequest): Promise<ScyllaResult<Role>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.updateRole({ ...input })).response,
      'Failed to update role.',
    );
  }

  public deleteRole(id: string): Promise<ScyllaResult<DeleteRoleResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._roles.deleteRole({ id })).response,
      'Failed to delete role.',
    );
  }

  public getEffectivePermissions(
    principalKind: PrincipalKind,
    principalId: string,
  ): Promise<ScyllaResult<GetEffectivePermissionsResponse>> {
    return ScyllaResult.tryAsync(
      async () =>
        (await this._roles.getEffectivePermissions({ principalKind, principalId })).response,
      'Failed to fetch effective permissions.',
    );
  }

  public listGrants(scope?: Scope, scopeId?: string): Promise<ScyllaResult<ListGrantsResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._grants.listGrants({ scope, scopeId })).response,
      'Failed to list grants.',
    );
  }

  public createGrant(input: CreateGrantRequest): Promise<ScyllaResult<Grant>> {
    return ScyllaResult.tryAsync(
      async () =>
        (
          await this._grants.createGrant({
            userId: input.userId,
            role: input.role,
            scope: input.scope,
            scopeId: input.scopeId,
            permission: input.permission,
          })
        ).response,
      'Failed to create grant.',
    );
  }

  public revokeGrant(id: string): Promise<ScyllaResult<RevokeGrantResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._grants.revokeGrant({ id })).response,
      'Failed to revoke grant.',
    );
  }

  public listGrantableRoles(scope?: Scope): Promise<ScyllaResult<ListGrantableRolesResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._grants.listGrantableRoles({ scope })).response,
      'Failed to list grantable roles.',
    );
  }

  public listAuthzVocabulary(): Promise<ScyllaResult<ListAuthzVocabularyResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._policies.listAuthzVocabulary({})).response,
      'Failed to load authz vocabulary.',
    );
  }
}

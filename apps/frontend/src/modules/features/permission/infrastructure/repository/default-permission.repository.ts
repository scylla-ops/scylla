import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { CreateGrantInput } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  PrincipalKind,
  PermissionScope,
} from '@/modules/features/permission/domain/models/permission.model.ts';
import type { GrpcPermissionRemoteDataSource } from '@/modules/features/permission/infrastructure/data/grpc-permission-remote.data-source.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/permission/domain/entities/grant.entity.ts';
import type { GrantableRoleEntity } from '@/modules/features/permission/domain/entities/grantable-role.entity.ts';
import type { EffectivePermissionsEntity } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import type { AuthzVocabularyEntity } from '@/modules/features/permission/domain/entities/authz-vocabulary.entity.ts';
import { GrpcRoleMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-role.mapper.ts';
import { GrpcGrantMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-grant.mapper.ts';
import { GrpcGrantableRoleMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-grantable-role.mapper.ts';
import { GrpcEffectivePermissionsMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-effective-permissions.mapper.ts';
import { GrpcAuthzVocabularyMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-authz-vocabulary.mapper.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';
export class DefaultPermissionRepository implements PermissionRepository {
  constructor(private readonly _dataSource: GrpcPermissionRemoteDataSource) {}

  // ── Roles ──────────────────────────────────────────────────────────────────

  public async listRoles(): Promise<ScyllaResult<RoleEntity[]>> {
    return (await this._dataSource.listRoles()).map(response =>
      response.roles.map(GrpcRoleMapper.toDomain),
    );
  }

  public async getRole(id: string): Promise<ScyllaResult<RoleEntity>> {
    return (await this._dataSource.getRoleById(id)).map(GrpcRoleMapper.toDomain);
  }

  public async createRole(role: RoleCreationData): Promise<ScyllaResult<RoleEntity>> {
    return (await this._dataSource.createRole(GrpcRoleMapper.toGrpcCreateRequest(role))).map(
      GrpcRoleMapper.toDomain,
    );
  }

  public async updateRole(role: RoleEntity): Promise<ScyllaResult<RoleEntity>> {
    return (
      await ScyllaResult.try(
        () => GrpcRoleMapper.toGrpcUpdateRequest(role),
        'Failed to map role to gRPC request',
      ).flatMapAsync(grpcRequest => this._dataSource.updateRole(grpcRequest))
    ).map(GrpcRoleMapper.toDomain);
  }

  public async deleteRole(id: string): Promise<ScyllaResult<boolean>> {
    return (await this._dataSource.deleteRole(id)).map(response => response.deleted);
  }

  // ── Introspection ──────────────────────────────────────────────────────────

  public async getEffectivePermissions(
    principalKind: PrincipalKind,
    principalId: string,
  ): Promise<ScyllaResult<EffectivePermissionsEntity>> {
    // Domain and gRPC PrincipalKind enums share identical numeric values.
    return (await this._dataSource.getEffectivePermissions(principalKind, principalId)).map(
      GrpcEffectivePermissionsMapper.toDomain,
    );
  }

  // ── Grants ─────────────────────────────────────────────────────────────────

  public async listGrants(
    scope?: PermissionScope,
    scopeId?: string,
  ): Promise<ScyllaResult<GrantEntity[]>> {
    return (
      await this._dataSource.listGrants(
        scope != null ? GrpcPermissionMapper.scopeToGrpc(scope) : undefined,
        scopeId,
      )
    ).map(response => response.grants.map(GrpcGrantMapper.toDomain));
  }

  public async createGrant(input: CreateGrantInput): Promise<ScyllaResult<GrantEntity>> {
    return (await this._dataSource.createGrant(GrpcGrantMapper.toGrpcCreateRequest(input))).map(
      GrpcGrantMapper.toDomain,
    );
  }

  public async revokeGrant(id: string): Promise<ScyllaResult<boolean>> {
    return (await this._dataSource.revokeGrant(id)).map(response => response.revoked);
  }

  public async listGrantableRoles(
    scope?: PermissionScope,
  ): Promise<ScyllaResult<GrantableRoleEntity[]>> {
    return (
      await this._dataSource.listGrantableRoles(
        scope != null ? GrpcPermissionMapper.scopeToGrpc(scope) : undefined,
      )
    ).map(response => response.roles.map(GrpcGrantableRoleMapper.toDomain));
  }

  // ── Vocabulary ─────────────────────────────────────────────────────────────

  public async listAuthzVocabulary(): Promise<ScyllaResult<AuthzVocabularyEntity>> {
    return (await this._dataSource.listAuthzVocabulary()).map(GrpcAuthzVocabularyMapper.toDomain);
  }
}

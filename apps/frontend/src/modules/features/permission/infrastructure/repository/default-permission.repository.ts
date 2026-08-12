import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { CreateGrantInput } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  PrincipalEntity,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { GrpcPermissionRemoteDataSource } from '@/modules/features/permission/infrastructure/data/grpc-permission-remote.data-source.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/permission/domain/entities/grant.entity.ts';
import type { GrantableRoleEntity } from '@/modules/features/permission/domain/entities/grantable-role.entity.ts';
import type { EffectivePermissionsEntity } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import type { PermissionVocabularyEntity } from '@/modules/features/permission/domain/entities/permission-vocabulary.entity.ts';
import { GrpcRoleMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-role.mapper.ts';
import { GrpcGrantMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-grant.mapper.ts';
import { GrpcGrantableRoleMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-grantable-role.mapper.ts';
import { GrpcEffectivePermissionsMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-effective-permissions.mapper.ts';
import { GrpcPermissionVocabularyMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission-vocabulary.mapper.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';
export class DefaultPermissionRepository implements PermissionRepository {
  constructor(private readonly _dataSource: GrpcPermissionRemoteDataSource) {}

  // ── Roles ──────────────────────────────────────────────────────────────────

  public async listRoles(): Promise<ScyllaResult<RoleEntity[]>> {
    return (await this._dataSource.listRoles()).map(roles => roles.map(GrpcRoleMapper.toDomain));
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

  public async deleteRole(id: string): Promise<ScyllaResult<void>> {
    return this._dataSource.deleteRole(id);
  }

  // ── Introspection ──────────────────────────────────────────────────────────

  public async getEffectivePermissions(
    principal: PrincipalEntity,
  ): Promise<ScyllaResult<EffectivePermissionsEntity>> {
    return (
      await ScyllaResult.try(
        () => GrpcPermissionMapper.principalRefToGrpc(principal),
        'Failed to map principal to gRPC request',
      ).flatMapAsync(ref => this._dataSource.getEffectivePermissions(ref))
    ).map(GrpcEffectivePermissionsMapper.toDomain);
  }

  public async getMyPermissions(): Promise<ScyllaResult<EffectivePermissionsEntity>> {
    return (await this._dataSource.getMyPermissions()).map(GrpcEffectivePermissionsMapper.toDomain);
  }

  // ── Grants ─────────────────────────────────────────────────────────────────

  public async listGrants(
    scope?: PermissionScope,
    scopeId?: string,
  ): Promise<ScyllaResult<GrantEntity[]>> {
    return (
      await ScyllaResult.try(
        // No scope filter → list every grant; otherwise bind the filter to the
        // scope's own id, as `ScopeRef` now carries both together.
        () =>
          scope != null ? GrpcPermissionMapper.scopeRefToGrpc(scope, scopeId ?? '') : undefined,
        'Failed to map scope to gRPC request',
      ).flatMapAsync(ref => this._dataSource.listGrants(ref))
    ).map(grants => grants.map(GrpcGrantMapper.toDomain));
  }

  public async createGrant(input: CreateGrantInput): Promise<ScyllaResult<GrantEntity>> {
    return (
      await ScyllaResult.try(
        () => GrpcGrantMapper.toGrpcCreateRequest(input),
        'Failed to map grant to gRPC request',
      ).flatMapAsync(request => this._dataSource.createGrant(request))
    ).map(GrpcGrantMapper.toDomain);
  }

  public async revokeGrant(id: string): Promise<ScyllaResult<void>> {
    return this._dataSource.revokeGrant(id);
  }

  public async listGrantableRoles(
    scope?: PermissionScope,
  ): Promise<ScyllaResult<GrantableRoleEntity[]>> {
    // The grantable-role catalog filters on the id-free `ScopeKind`.
    return (
      await this._dataSource.listGrantableRoles(
        scope != null ? GrpcPermissionMapper.scopeToGrpc(scope) : undefined,
      )
    ).map(roles => roles.map(GrpcGrantableRoleMapper.toDomain));
  }

  // ── Vocabulary ─────────────────────────────────────────────────────────────

  public async listPermissionVocabulary(): Promise<ScyllaResult<PermissionVocabularyEntity>> {
    return (await this._dataSource.listPermissionVocabulary()).map(
      GrpcPermissionVocabularyMapper.toDomain,
    );
  }
}

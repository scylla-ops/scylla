import type { EffectiveScope, GetEffectivePermissionsResponse } from '@/generated/permission.ts';
import type {
  EffectivePermissionsEntity,
  EffectiveScopeEntity,
} from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcEffectivePermissionsMapper {
  public static scopeToDomain(grpcScope: EffectiveScope): EffectiveScopeEntity {
    return {
      scope: GrpcPermissionMapper.scopeToDomain(grpcScope.scope),
      scopeId: grpcScope.scopeId,
      fullControl: grpcScope.fullControl,
      permissions: grpcScope.permissions.map(GrpcPermissionMapper.toDomain),
    };
  }

  public static toDomain(
    grpcResponse: GetEffectivePermissionsResponse,
  ): EffectivePermissionsEntity {
    return {
      scopes: grpcResponse.scopes.map(GrpcEffectivePermissionsMapper.scopeToDomain),
    };
  }
}


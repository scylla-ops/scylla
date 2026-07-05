import type { EffectiveScope, GetEffectivePermissionsResponse } from '@/generated/permission.ts';
import type {
  EffectivePermissionsEntity,
  EffectiveScopeEntity,
} from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcEffectivePermissionsMapper {
  public static scopeToDomain(grpcScope: EffectiveScope): EffectiveScopeEntity {
    // `access` is a oneof: full control (no list) or an explicit permission set.
    return {
      scope: GrpcPermissionMapper.scopeToDomain(grpcScope.scope),
      scopeId: grpcScope.scopeId,
      fullControl: grpcScope.access.oneofKind === 'fullControl',
      permissions:
        grpcScope.access.oneofKind === 'restricted'
          ? grpcScope.access.restricted.permissions.map(GrpcPermissionMapper.toDomain)
          : [],
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


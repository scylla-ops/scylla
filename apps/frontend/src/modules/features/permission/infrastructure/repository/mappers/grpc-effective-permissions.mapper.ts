import type { EffectiveScope } from '@/generated/scylla/authz/v1/role.ts';
import type {
  EffectivePermissionsEntity,
  EffectiveScopeEntity,
} from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcEffectivePermissionsMapper {
  public static scopeToDomain(grpcScope: EffectiveScope): EffectiveScopeEntity {
    const { scope, scopeId } = GrpcPermissionMapper.scopeRefToDomain(grpcScope.scope);

    return {
      scope,
      scopeId,
      access: GrpcPermissionMapper.accessToDomain(grpcScope.access),
    };
  }

  public static toDomain(grpcScopes: EffectiveScope[]): EffectivePermissionsEntity {
    return {
      scopes: grpcScopes.map(GrpcEffectivePermissionsMapper.scopeToDomain),
    };
  }
}

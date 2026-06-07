import type { GrantableRole } from '@/generated/permission.ts';
import type { GrantableRoleEntity } from '@/modules/features/permission/domain/entities/grantable-role.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcGrantableRoleMapper {
  public static toDomain(grpcRole: GrantableRole): GrantableRoleEntity {
    return {
      name: grpcRole.name,
      scope: GrpcPermissionMapper.scopeToDomain(grpcRole.scope),
      kind: grpcRole.kind,
      description: grpcRole.description,
    };
  }
}


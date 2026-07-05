import type { CreateRoleRequest, Role, UpdateRoleRequest } from '@/generated/permission.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcRoleMapper {
  public static toDomain(grpcRole: Role): RoleEntity {
    // `access` is a oneof: full control (no list) or an explicit permission set.
    return {
      id: grpcRole.id,
      key: grpcRole.key,
      name: grpcRole.name,
      description: grpcRole.description,
      scope: GrpcPermissionMapper.scopeToDomain(grpcRole.scope),
      builtin: grpcRole.builtin,
      fullControl: grpcRole.access.oneofKind === 'fullControl',
      permissions:
        grpcRole.access.oneofKind === 'restricted'
          ? grpcRole.access.restricted.permissions.map(GrpcPermissionMapper.toDomain)
          : [],
    };
  }

  public static toGrpcCreateRequest(data: RoleCreationData): CreateRoleRequest {
    return {
      name: data.name,
      description: data.description,
      scope: GrpcPermissionMapper.scopeToGrpc(data.scope),
      access: data.fullControl
        ? { oneofKind: 'fullControl', fullControl: {} }
        : {
            oneofKind: 'restricted',
            restricted: { permissions: data.permissions.map(GrpcPermissionMapper.toGrpc) },
          },
    };
  }

  public static toGrpcUpdateRequest(role: RoleEntity): UpdateRoleRequest {
    return {
      id: role.id,
      name: role.name,
      description: role.description,
      access: role.fullControl
        ? { oneofKind: 'fullControl', fullControl: {} }
        : {
            oneofKind: 'restricted',
            restricted: { permissions: role.permissions.map(GrpcPermissionMapper.toGrpc) },
          },
    };
  }
}

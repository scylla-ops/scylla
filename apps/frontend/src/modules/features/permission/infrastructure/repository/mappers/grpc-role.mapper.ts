import type { CreateRoleRequest, Role, UpdateRoleRequest } from '@/generated/permission.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcRoleMapper {
  public static toDomain(grpcRole: Role): RoleEntity {
    return {
      ...grpcRole,
      permissions: grpcRole.permissions.map(GrpcPermissionMapper.toDomain),
      scope: GrpcPermissionMapper.scopeToDomain(grpcRole.scope),
    };
  }

  public static toGrpcCreateRequest(data: RoleCreationData): CreateRoleRequest {
    return {
      fullControl: data.fullControl,
      name: data.name,
      permissions: data.permissions.map(GrpcPermissionMapper.toGrpc),
      scope: GrpcPermissionMapper.scopeToGrpc(data.scope),
      description: data.description,
    };
  }

  public static toGrpcUpdateRequest(role: RoleEntity): UpdateRoleRequest {
    return {
      fullControl: role.fullControl,
      name: role.name,
      permissions: role.permissions.map(GrpcPermissionMapper.toGrpc),
      id: role.id,
      description: role.description,
    };
  }
}

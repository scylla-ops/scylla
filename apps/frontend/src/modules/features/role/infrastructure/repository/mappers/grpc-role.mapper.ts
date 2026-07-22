import type {
  CreateRoleRequest,
  Role,
  UpdateRoleRequest,
} from '@/generated/scylla/authz/v1/role.ts';
import type {
  RoleCreationData,
  RoleEntity,
  RoleOrigin,
} from '@/modules/features/role/domain/entities/role.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/role/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcRoleMapper {
  /**
   * `origin` is a oneof: a builtin role carries its stable key, a custom one
   * the organization that owns it. An arm this build does not know surfaces as
   * `unknown` rather than being read as "custom".
   */
  private static originToDomain(grpcRole: Role): RoleOrigin {
    switch (grpcRole.origin.oneofKind) {
      case 'builtin':
        return { kind: 'builtin', key: grpcRole.origin.builtin.key };
      case 'custom':
        return {
          kind: 'custom',
          ownerOrganizationId: grpcRole.origin.custom.ownerOrganizationId?.value,
        };
      default:
        return { kind: 'unknown' };
    }
  }

  public static toDomain(grpcRole: Role): RoleEntity {
    return {
      id: grpcRole.roleId?.value ?? '',
      name: grpcRole.name,
      description: grpcRole.description,
      scope: GrpcPermissionMapper.scopeToDomain(grpcRole.scopeKind),
      origin: GrpcRoleMapper.originToDomain(grpcRole),
      access: GrpcPermissionMapper.accessToDomain(grpcRole.access),
    };
  }

  public static toGrpcCreateRequest(data: RoleCreationData): CreateRoleRequest {
    return {
      name: data.name,
      description: data.description,
      scopeKind: GrpcPermissionMapper.scopeToGrpc(data.scope),
      access: GrpcPermissionMapper.accessToGrpc(data.access),
    };
  }

  public static toGrpcUpdateRequest(role: RoleEntity): UpdateRoleRequest {
    if (role.access.kind === 'unknown') {
      throw new Error(
        'This role uses an access mode this version does not understand; it cannot be updated here.',
      );
    }

    return {
      roleId: { value: role.id },
      name: role.name,
      description: role.description,
      access: GrpcPermissionMapper.accessToGrpc(role.access),
    };
  }
}

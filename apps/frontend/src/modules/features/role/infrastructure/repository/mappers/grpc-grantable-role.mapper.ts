import { type GrantableRole, RoleKind as GrpcRoleKind } from '@/generated/scylla/authz/v1/grant.ts';
import type { GrantableRoleEntity } from '@/modules/features/role/domain/entities/grantable-role.entity.ts';
import { RoleKind } from '@/modules/features/role/domain/structs/permission.struct.ts';
import { GrpcPermissionMapper } from '@/modules/features/role/infrastructure/repository/mappers/grpc-permission.mapper.ts';

/**
 * Wire `RoleKind` → domain `RoleKind`, arm by arm. The two enums happen to share
 * their numeric values today, but they are declared independently: a `switch`
 * turns a future divergence into a compile error instead of a silent mismap.
 */
function roleKindToDomain(kind: GrpcRoleKind): RoleKind {
  switch (kind) {
    case GrpcRoleKind.ADMIN:
      return RoleKind.ADMIN;
    case GrpcRoleKind.AGENT:
      return RoleKind.AGENT;
    case GrpcRoleKind.UNSPECIFIED:
      return RoleKind.UNSPECIFIED;
  }
}

export class GrpcGrantableRoleMapper {
  public static toDomain(grpcRole: GrantableRole): GrantableRoleEntity {
    return {
      roleId: grpcRole.roleId?.value ?? '',
      scope: GrpcPermissionMapper.scopeToDomain(grpcRole.scopeKind),
      kind: roleKindToDomain(grpcRole.kind),
      description: grpcRole.description,
    };
  }
}

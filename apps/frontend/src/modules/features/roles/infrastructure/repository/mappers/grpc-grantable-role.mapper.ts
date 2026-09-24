import { type GrantableRole, RoleKind as GrpcRoleKind } from '@/generated/scylla/authz/v1/grant.ts';
import type { GrantableRoleEntity } from '@/modules/features/roles/domain/entities/grantable-role.entity.ts';
import { RoleKind } from '@platform/authz';
import { GrpcPermissionMapper } from '@/modules/features/roles/infrastructure/repository/mappers/grpc-permission.mapper.ts';

/** A `switch`, so a divergence of the two enums fails to compile. */
function roleKindToDomain(kind: GrpcRoleKind): RoleKind {
  switch (kind) {
    case GrpcRoleKind.ADMIN:
      return RoleKind.ADMIN;
    case GrpcRoleKind.AGENT:
      return RoleKind.AGENT;
    case GrpcRoleKind.MEMBER:
      return RoleKind.MEMBER;
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

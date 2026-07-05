import type { CreateGrantRequest, Grant } from '@/generated/permission.ts';
import type { GrantEntity } from '@/modules/features/permission/domain/entities/grant.entity.ts';
import type { CreateGrantInput } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcGrantMapper {
  /**
   * Maps a gRPC {@link Grant} to a domain {@link GrantEntity}.
   * The `UserId` wrapper is unwrapped to a plain string.
   */
  public static toDomain(grpcGrant: Grant): GrantEntity {
    // `grantType` is a oneof: a role or a single permission, never both.
    return {
      id: grpcGrant.id,
      userId: grpcGrant.userId?.value,
      role: grpcGrant.grantType.oneofKind === 'role' ? grpcGrant.grantType.role : '',
      scope: GrpcPermissionMapper.scopeToDomain(grpcGrant.scope),
      scopeId: grpcGrant.scopeId,
      permission:
        grpcGrant.grantType.oneofKind === 'permission'
          ? GrpcPermissionMapper.toDomain(grpcGrant.grantType.permission)
          : undefined,
    };
  }

  /**
   * Maps a domain {@link CreateGrantInput} to the gRPC
   * {@link CreateGrantRequest}.
   * The plain string `userId` is wrapped in the required `UserId` message.
   */
  public static toGrpcCreateRequest(input: CreateGrantInput): CreateGrantRequest {
    // A permission set → a single-permission grant; otherwise a role grant.
    return {
      userId: input.userId != null ? { value: input.userId } : undefined,
      scope: GrpcPermissionMapper.scopeToGrpc(input.scope),
      scopeId: input.scopeId,
      grantType:
        input.permission != null
          ? { oneofKind: 'permission', permission: GrpcPermissionMapper.toGrpc(input.permission) }
          : { oneofKind: 'role', role: input.role },
    };
  }
}


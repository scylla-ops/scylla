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
    return {
      id: grpcGrant.id,
      userId: grpcGrant.userId?.value,
      role: grpcGrant.role,
      scope: GrpcPermissionMapper.scopeToDomain(grpcGrant.scope),
      scopeId: grpcGrant.scopeId,
      permission:
        grpcGrant.permission != null
          ? GrpcPermissionMapper.toDomain(grpcGrant.permission)
          : undefined,
    };
  }

  /**
   * Maps a domain {@link CreateGrantInput} to the gRPC
   * {@link CreateGrantRequest}.
   * The plain string `userId` is wrapped in the required `UserId` message.
   */
  public static toGrpcCreateRequest(input: CreateGrantInput): CreateGrantRequest {
    return {
      userId: input.userId != null ? { value: input.userId } : undefined,
      role: input.role,
      scope: GrpcPermissionMapper.scopeToGrpc(input.scope),
      scopeId: input.scopeId,
      permission:
        input.permission != null ? GrpcPermissionMapper.toGrpc(input.permission) : undefined,
    };
  }
}


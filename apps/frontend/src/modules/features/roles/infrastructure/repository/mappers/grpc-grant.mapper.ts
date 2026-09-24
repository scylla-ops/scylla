import type { CreateGrantRequest, Grant } from '@/generated/scylla/authz/v1/grant.ts';
import type { GrantEntity } from '@/modules/features/roles/domain/entities/grant.entity.ts';
import type { CreateGrantInput } from '@/modules/features/roles/domain/repository/permission.repository.ts';
import { GrpcPermissionMapper } from '@/modules/features/roles/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcGrantMapper {
  public static toDomain(grpcGrant: Grant): GrantEntity {
    const { scope, scopeId } = GrpcPermissionMapper.scopeRefToDomain(grpcGrant.scope);

    return {
      id: grpcGrant.grantId?.value ?? '',
      principal: GrpcPermissionMapper.principalRefToDomain(grpcGrant.principal),
      roleId: grpcGrant.role?.value ?? '',
      scope,
      scopeId,
    };
  }

  /** Throws when the principal or the scope is unspecified. */
  public static toGrpcCreateRequest(input: CreateGrantInput): CreateGrantRequest {
    return {
      principal: GrpcPermissionMapper.principalRefToGrpc(input.principal),
      scope: GrpcPermissionMapper.scopeRefToGrpc(input.scope, input.scopeId),
      role: { value: input.roleId },
    };
  }
}

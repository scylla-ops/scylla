import type { CreateGrantRequest, Grant } from '@/generated/scylla/authz/v1/grant.ts';
import type {
  GrantEntity,
  GrantTarget,
} from '@/modules/features/permission/domain/entities/grant.entity.ts';
import type { CreateGrantInput } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcGrantMapper {
  /**
   * `target` is a oneof: a role or a single permission, never both. An arm this
   * build does not know surfaces as `unknown` rather than as an empty role.
   */
  private static targetToDomain(grpcGrant: Grant): GrantTarget {
    switch (grpcGrant.target.oneofKind) {
      case 'role':
        return { kind: 'role', roleId: grpcGrant.target.role.value };
      case 'permission':
        return {
          kind: 'permission',
          permission: GrpcPermissionMapper.toDomain(grpcGrant.target.permission),
        };
      default:
        return { kind: 'unknown' };
    }
  }

  /**
   * Maps a gRPC {@link Grant} to a domain {@link GrantEntity}.
   * The id wrappers are unwrapped to plain strings, and the `PrincipalRef`
   * oneof is flattened to a (kind, id) pair — a grant may target a user or an
   * app.
   */
  public static toDomain(grpcGrant: Grant): GrantEntity {
    const { scope, scopeId } = GrpcPermissionMapper.scopeRefToDomain(grpcGrant.scope);

    return {
      id: grpcGrant.grantId?.value ?? '',
      principal: GrpcPermissionMapper.principalRefToDomain(grpcGrant.principal),
      target: GrpcGrantMapper.targetToDomain(grpcGrant),
      scope,
      scopeId,
    };
  }

  /**
   * Maps a domain {@link CreateGrantInput} to the gRPC
   * {@link CreateGrantRequest}. Throws when the principal or the scope was left
   * unspecified — the backend rejects those anyway.
   */
  public static toGrpcCreateRequest(input: CreateGrantInput): CreateGrantRequest {
    return {
      principal: GrpcPermissionMapper.principalRefToGrpc(input.principal),
      scope: GrpcPermissionMapper.scopeRefToGrpc(input.scope, input.scopeId),
      target:
        input.target.kind === 'permission'
          ? {
              oneofKind: 'permission',
              permission: GrpcPermissionMapper.toGrpc(input.target.permission),
            }
          : { oneofKind: 'role', role: { value: input.target.roleId } },
    };
  }
}

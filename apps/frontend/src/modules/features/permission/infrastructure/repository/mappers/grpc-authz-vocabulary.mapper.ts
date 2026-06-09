import type { AuthzAction, ListAuthzVocabularyResponse } from '@/generated/permission.ts';
import type {
  AuthzActionEntity,
  AuthzVocabularyEntity,
} from '@/modules/features/permission/domain/entities/authz-vocabulary.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcAuthzVocabularyMapper {
  public static actionToDomain(grpcAction: AuthzAction): AuthzActionEntity {
    return {
      permission: GrpcPermissionMapper.toDomain(grpcAction.permission),
      minScope: GrpcPermissionMapper.scopeToDomain(grpcAction.minScope),
    };
  }

  public static toDomain(grpcResponse: ListAuthzVocabularyResponse): AuthzVocabularyEntity {
    return {
      actions: grpcResponse.actions.map(GrpcAuthzVocabularyMapper.actionToDomain),
    };
  }
}


import type { AuthzAction } from '@/generated/scylla/authz/v1/permission.ts';
import type {
  PermissionActionEntity,
  PermissionVocabularyEntity,
} from '@/modules/features/permission/domain/entities/permission-vocabulary.entity.ts';
import { GrpcPermissionMapper } from '@/modules/features/permission/infrastructure/repository/mappers/grpc-permission.mapper.ts';

export class GrpcPermissionVocabularyMapper {
  public static actionToDomain(grpcAction: AuthzAction): PermissionActionEntity {
    return {
      permission: GrpcPermissionMapper.toDomain(grpcAction.permission),
      minScope: GrpcPermissionMapper.scopeToDomain(grpcAction.minScope),
    };
  }

  public static toDomain(grpcActions: AuthzAction[]): PermissionVocabularyEntity {
    return {
      actions: grpcActions.map(GrpcPermissionVocabularyMapper.actionToDomain),
    };
  }
}

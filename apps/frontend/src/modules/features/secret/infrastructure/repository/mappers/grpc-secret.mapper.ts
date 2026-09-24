import type { Secret as ProtoSecret } from '@/generated/scylla/secret/v1/secret.ts';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcSecretMapper {
  static toDomain(s: ProtoSecret): SecretEntity {
    return {
      id: idValue(s.secretId),
      projectId: idValue(s.projectId),
      name: s.name,
      description: s.description,
      createdAt: timestampToIso(s.createdAt),
      updatedAt: timestampToIso(s.updatedAt),
    };
  }
}

import type { Secret as ProtoSecret } from '@/generated/secret.ts';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

/** Maps gRPC Secret messages to the domain Secret model (never carries a value). */
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

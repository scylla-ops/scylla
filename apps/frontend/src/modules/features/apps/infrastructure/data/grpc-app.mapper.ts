import type {
  App as ProtoApp,
  AppSecret as ProtoAppSecret,
} from '@/generated/scylla/app/v1/app.ts';
import type {
  AppEntity,
  AppSecretEntity,
} from '@/modules/features/apps/domain/entities/app.entity.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

/** Maps gRPC App messages to the domain App model. */
export class GrpcAppMapper {
  static toDomain(a: ProtoApp): AppEntity {
    return {
      id: idValue(a.appId),
      organizationId: idValue(a.organizationId),
      name: a.name,
      isActive: a.isActive,
      createdAt: timestampToIso(a.createdAt),
      updatedAt: timestampToIso(a.updatedAt),
    };
  }

  static secretToDomain(s: ProtoAppSecret): AppSecretEntity {
    return {
      id: idValue(s.appSecretId),
      appId: idValue(s.appId),
      label: s.label,
      enabled: s.enabled,
      createdAt: timestampToIso(s.createdAt),
      updatedAt: timestampToIso(s.updatedAt),
    };
  }
}

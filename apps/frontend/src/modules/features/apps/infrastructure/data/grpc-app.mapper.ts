import type { App as ProtoApp, AppSecret as ProtoAppSecret } from '@/generated/app.ts';
import type { App, AppSecret } from '@/modules/features/apps/domain/models/app.model.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

/** Maps gRPC App messages to the domain App model. */
export class GrpcAppMapper {
  static toDomain(a: ProtoApp): App {
    return {
      id: idValue(a.id),
      organizationId: idValue(a.organizationId),
      name: a.name,
      isActive: a.isActive,
      createdAt: timestampToIso(a.createdAt),
      updatedAt: timestampToIso(a.updatedAt),
    };
  }

  static secretToDomain(s: ProtoAppSecret): AppSecret {
    return {
      id: idValue(s.id),
      appId: idValue(s.appId),
      label: s.label,
      enabled: s.enabled,
      createdAt: timestampToIso(s.createdAt),
      updatedAt: timestampToIso(s.updatedAt),
    };
  }
}

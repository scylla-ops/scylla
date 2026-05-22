import type { App as ProtoApp, AppSecret as ProtoAppSecret } from '@/generated/app.ts';
import type { App, AppSecret } from '@/modules/features/apps/domain/models/app.model.ts';

/** Maps gRPC App messages to the domain App model. */
export class GrpcAppMapper {
  static toDomain(a: ProtoApp): App {
    return {
      id: a.id,
      organizationId: a.organizationId,
      name: a.name,
      isActive: a.isActive,
      createdAt: a.createdAt,
      updatedAt: a.updatedAt,
    };
  }

  static secretToDomain(s: ProtoAppSecret): AppSecret {
    return {
      id: s.id,
      appId: s.appId,
      label: s.label,
      enabled: s.enabled,
      createdAt: s.createdAt,
      updatedAt: s.updatedAt,
    };
  }
}

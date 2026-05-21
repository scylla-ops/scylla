import type { App as ProtoApp } from '@/generated/app.ts';
import type { App } from '@/modules/features/apps/domain/models/app.model.ts';

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
}

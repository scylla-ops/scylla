import type {
  App as ProtoApp,
  AppSecret as ProtoAppSecret,
  CreateAppResponse,
  CreateAppSecretResponse,
  ListAppSecretsResponse,
  ListAppsResponse,
} from '@/generated/scylla/app/v1/app.ts';
import type {
  AppEntity,
  AppSecretEntity,
} from '@/modules/features/apps/domain/entities/app.entity.ts';
import type {
  CreatedApp,
  CreatedAppSecret,
} from '@/modules/features/apps/domain/structs/app.struct.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

/**
 * Every app RPC answers with a `XxxResponse` wrapper holding the entity in
 * field 1. The unwrapping lives here, so an absent entity means the server
 * answered with a shape this build cannot read.
 */
function requireApp(app: ProtoApp | undefined): ProtoApp {
  if (!app) throw new Error('Server returned no app.');
  return app;
}

function requireAppSecret(appSecret: ProtoAppSecret | undefined): ProtoAppSecret {
  if (!appSecret) throw new Error('Server returned no app secret.');
  return appSecret;
}

/** Maps gRPC App messages to the domain App entities. */
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

  /** For the single-app responses (`GetApp`, `SetAppActive`). */
  static toDomainFromResponse(response: { app?: ProtoApp }): AppEntity {
    return GrpcAppMapper.toDomain(requireApp(response.app));
  }

  static toDomainList(response: ListAppsResponse): AppEntity[] {
    return response.apps.map(GrpcAppMapper.toDomain);
  }

  static toCreatedApp(response: CreateAppResponse): CreatedApp {
    return {
      app: GrpcAppMapper.toDomain(requireApp(response.app)),
      secret: response.secret,
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

  /** For the single-secret responses (`SetAppSecretEnabled`). */
  static secretToDomainFromResponse(response: { appSecret?: ProtoAppSecret }): AppSecretEntity {
    return GrpcAppMapper.secretToDomain(requireAppSecret(response.appSecret));
  }

  static secretToDomainList(response: ListAppSecretsResponse): AppSecretEntity[] {
    return response.appSecrets.map(GrpcAppMapper.secretToDomain);
  }

  static toCreatedAppSecret(response: CreateAppSecretResponse): CreatedAppSecret {
    return {
      credential: GrpcAppMapper.secretToDomain(requireAppSecret(response.appSecret)),
      secret: response.secret,
    };
  }
}

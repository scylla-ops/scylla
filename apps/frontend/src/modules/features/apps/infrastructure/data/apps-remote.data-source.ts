import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { AppServiceClient } from '@/generated/scylla/app/v1/app.client.ts';
import type {
  AppEntity,
  AppSecretEntity,
} from '@/modules/features/apps/domain/entities/app.entity.ts';
import type {
  CreatedApp,
  CreatedAppSecret,
} from '@/modules/features/apps/domain/structs/app.struct.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';
import { GrpcAppMapper } from './grpc-app.mapper.ts';

/** Remote data source for apps over gRPC. Shares the repository contract. */
export type AppsRemoteDataSource = AppsRepository;

export class AppsRemoteDataSourceImpl implements AppsRemoteDataSource {
  constructor(private grpcTransport: CoreGrpcTransport) {}

  listApps(organizationId: string): Promise<ScyllaResult<AppEntity[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.listApps({ organizationId: wrapId(organizationId) }).response;
      return response.apps.map(app => GrpcAppMapper.toDomain(app));
    }, 'Failed to list apps');
  }

  getApp(appId: string): Promise<ScyllaResult<AppEntity>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.getApp({ appId: wrapId(appId) }).response;
      if (!response.app) throw new Error('GetApp returned no app');
      return GrpcAppMapper.toDomain(response.app);
    }, 'Failed to fetch app');
  }

  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.createApp({ organizationId: wrapId(organizationId), name })
        .response;
      if (!response.app) throw new Error('CreateApp returned no app');
      return { app: GrpcAppMapper.toDomain(response.app), secret: response.secret };
    }, 'Failed to create app');
  }

  deleteApp(appId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      await client.deleteApp({ appId: wrapId(appId) }).response;
    }, 'Failed to delete app');
  }

  setAppActive(appId: string, active: boolean): Promise<ScyllaResult<AppEntity>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.setAppActive({ appId: wrapId(appId), isActive: active })
        .response;
      if (!response.app) throw new Error('SetAppActive returned no app');
      return GrpcAppMapper.toDomain(response.app);
    }, 'Failed to update app');
  }

  listAppSecrets(appId: string): Promise<ScyllaResult<AppSecretEntity[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.listAppSecrets({ appId: wrapId(appId) }).response;
      return response.appSecrets.map(s => GrpcAppMapper.secretToDomain(s));
    }, 'Failed to list app secrets');
  }

  createAppSecret(appId: string, label: string): Promise<ScyllaResult<CreatedAppSecret>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.createAppSecret({ appId: wrapId(appId), label }).response;
      if (!response.appSecret) throw new Error('CreateAppSecret returned no secret metadata');
      return {
        credential: GrpcAppMapper.secretToDomain(response.appSecret),
        secret: response.secret,
      };
    }, 'Failed to create app secret');
  }

  revokeAppSecret(secretId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      await client.revokeAppSecret({ appSecretId: wrapId(secretId) }).response;
    }, 'Failed to revoke app secret');
  }

  setAppSecretEnabled(secretId: string, enabled: boolean): Promise<ScyllaResult<AppSecretEntity>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.setAppSecretEnabled({
        appSecretId: wrapId(secretId),
        enabled,
      }).response;
      if (!response.appSecret) throw new Error('SetAppSecretEnabled returned no secret metadata');
      return GrpcAppMapper.secretToDomain(response.appSecret);
    }, 'Failed to update app secret');
  }
}

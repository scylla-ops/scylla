import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { AppServiceClient } from '@/generated/app.client.ts';
import type {
  App,
  AppSecret,
  CreatedApp,
  CreatedAppSecret,
} from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';
import { GrpcAppMapper } from './grpc-app.mapper.ts';

/** Remote data source for apps over gRPC. Shares the repository contract. */
export type AppsRemoteDataSource = AppsRepository;

export class AppsRemoteDataSourceImpl implements AppsRemoteDataSource {
  constructor(private grpcTransport: CoreGrpcTransport) {}

  listApps(organizationId: string): Promise<ScyllaResult<App[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.listApps({ organizationId: wrapId(organizationId) }).response;
      return response.apps.map(app => GrpcAppMapper.toDomain(app));
    }, 'Failed to list apps');
  }

  getApp(appId: string): Promise<ScyllaResult<App>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.getApp({ id: wrapId(appId) }).response;
      return GrpcAppMapper.toDomain(response);
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

  deleteApp(appId: string): Promise<ScyllaResult<boolean>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.deleteApp({ id: wrapId(appId) }).response;
      return response.deleted;
    }, 'Failed to delete app');
  }

  setAppActive(appId: string, active: boolean): Promise<ScyllaResult<App>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.setAppActive({ appId: wrapId(appId), active }).response;
      return GrpcAppMapper.toDomain(response);
    }, 'Failed to update app');
  }

  listAppSecrets(appId: string): Promise<ScyllaResult<AppSecret[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.listAppSecrets({ appId: wrapId(appId) }).response;
      return response.secrets.map(s => GrpcAppMapper.secretToDomain(s));
    }, 'Failed to list app secrets');
  }

  createAppSecret(appId: string, label: string): Promise<ScyllaResult<CreatedAppSecret>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.createAppSecret({ appId: wrapId(appId), label }).response;
      if (!response.credential) throw new Error('CreateAppSecret returned no credential');
      return {
        credential: GrpcAppMapper.secretToDomain(response.credential),
        secret: response.secret,
      };
    }, 'Failed to create app secret');
  }

  revokeAppSecret(secretId: string): Promise<ScyllaResult<boolean>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.revokeAppSecret({ secretId: wrapId(secretId) }).response;
      return response.deleted;
    }, 'Failed to revoke app secret');
  }

  setAppSecretEnabled(secretId: string, enabled: boolean): Promise<ScyllaResult<AppSecret>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.setAppSecretEnabled({ secretId: wrapId(secretId), enabled })
        .response;
      return GrpcAppMapper.secretToDomain(response);
    }, 'Failed to update app secret');
  }
}

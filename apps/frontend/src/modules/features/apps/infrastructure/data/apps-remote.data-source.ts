import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { AppServiceClient } from '@/generated/app.client.ts';
import type { App, CreatedApp } from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { GrpcAppMapper } from './grpc-app.mapper.ts';

/** Remote data source for apps over gRPC. Shares the repository contract. */
export type AppsRemoteDataSource = AppsRepository;

export class AppsRemoteDataSourceImpl implements AppsRemoteDataSource {
  constructor(private grpcTransport: CoreGrpcTransport) {}

  listApps(organizationId: string): Promise<ScyllaResult<App[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.listApps({ organizationId }).response;
      return response.apps.map(app => GrpcAppMapper.toDomain(app));
    }, 'Failed to list apps');
  }

  getApp(appId: string): Promise<ScyllaResult<App>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.getApp({ id: appId }).response;
      return GrpcAppMapper.toDomain(response);
    }, 'Failed to fetch app');
  }

  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.createApp({ organizationId, name }).response;
      if (!response.app) throw new Error('CreateApp returned no app');
      return { app: GrpcAppMapper.toDomain(response.app), secret: response.secret };
    }, 'Failed to create app');
  }

  deleteApp(appId: string): Promise<ScyllaResult<boolean>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AppServiceClient(this.grpcTransport.getTransport());
      const response = await client.deleteApp({ id: appId }).response;
      return response.deleted;
    }, 'Failed to delete app');
  }
}

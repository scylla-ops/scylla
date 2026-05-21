import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';
import type { AppsRemoteDataSource } from '@/modules/features/apps/infrastructure/data/apps-remote.data-source.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { App, CreatedApp } from '@/modules/features/apps/domain/models/app.model.ts';

/** AppsRepository backed by the remote (gRPC) data source. */
export class DefaultAppsRepository implements AppsRepository {
  constructor(private remoteDataSource: AppsRemoteDataSource) {}

  listApps(organizationId: string): Promise<ScyllaResult<App[]>> {
    return this.remoteDataSource.listApps(organizationId);
  }

  getApp(appId: string): Promise<ScyllaResult<App>> {
    return this.remoteDataSource.getApp(appId);
  }

  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>> {
    return this.remoteDataSource.createApp(organizationId, name);
  }

  deleteApp(appId: string): Promise<ScyllaResult<boolean>> {
    return this.remoteDataSource.deleteApp(appId);
  }
}

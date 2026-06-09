import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';
import type { AppsRemoteDataSource } from '@/modules/features/apps/infrastructure/data/apps-remote.data-source.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  App,
  AppSecret,
  CreatedApp,
  CreatedAppSecret,
} from '@/modules/features/apps/domain/models/app.model.ts';

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

  setAppActive(appId: string, active: boolean): Promise<ScyllaResult<App>> {
    return this.remoteDataSource.setAppActive(appId, active);
  }

  listAppSecrets(appId: string): Promise<ScyllaResult<AppSecret[]>> {
    return this.remoteDataSource.listAppSecrets(appId);
  }

  createAppSecret(appId: string, label: string): Promise<ScyllaResult<CreatedAppSecret>> {
    return this.remoteDataSource.createAppSecret(appId, label);
  }

  revokeAppSecret(secretId: string): Promise<ScyllaResult<boolean>> {
    return this.remoteDataSource.revokeAppSecret(secretId);
  }

  setAppSecretEnabled(secretId: string, enabled: boolean): Promise<ScyllaResult<AppSecret>> {
    return this.remoteDataSource.setAppSecretEnabled(secretId, enabled);
  }
}

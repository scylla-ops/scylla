import type {
  CreateAppResponse,
  CreateAppSecretResponse,
  GetAppResponse,
  ListAppSecretsResponse,
  ListAppsResponse,
  SetAppActiveResponse,
  SetAppSecretEnabledResponse,
} from '@/generated/scylla/app/v1/app.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

/**
 * Remote data source for apps. It only speaks gRPC: it returns the raw proto
 * responses and leaves the domain mapping to the repository.
 */
export interface AppsRemoteDataSource {
  listApps(organizationId: string): Promise<ScyllaResult<ListAppsResponse>>;
  getApp(appId: string): Promise<ScyllaResult<GetAppResponse>>;
  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreateAppResponse>>;
  deleteApp(appId: string): Promise<ScyllaResult<void>>;
  setAppActive(appId: string, isActive: boolean): Promise<ScyllaResult<SetAppActiveResponse>>;

  listAppSecrets(appId: string): Promise<ScyllaResult<ListAppSecretsResponse>>;
  createAppSecret(appId: string, label: string): Promise<ScyllaResult<CreateAppSecretResponse>>;
  revokeAppSecret(appSecretId: string): Promise<ScyllaResult<void>>;
  setAppSecretEnabled(
    appSecretId: string,
    enabled: boolean,
  ): Promise<ScyllaResult<SetAppSecretEnabledResponse>>;
}

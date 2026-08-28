import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import type {
  CreateAppResponse,
  CreateAppSecretResponse,
  GetAppResponse,
  ListAppSecretsResponse,
  ListAppsResponse,
  SetAppActiveResponse,
  SetAppSecretEnabledResponse,
} from '@/generated/scylla/app/v1/app.ts';
import type { AppsRemoteDataSource } from '@/modules/features/apps/infrastructure/repository/data-sources/apps-remote.data-source.ts';
import { AppServiceClient } from '@/generated/scylla/app/v1/app.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcAppsRemoteDataSource implements AppsRemoteDataSource {
  private readonly _appClient: AppServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._appClient = new AppServiceClient(transport.getTransport());
  }

  public listApps(organizationId: string): Promise<ScyllaResult<ListAppsResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.listApps({
        organizationId: wrapId(organizationId),
      });
      return response;
    }, 'Failed to list apps.');
  }

  public getApp(appId: string): Promise<ScyllaResult<GetAppResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.getApp({ appId: wrapId(appId) });
      return response;
    }, 'Failed to fetch app.');
  }

  public createApp(organizationId: string, name: string): Promise<ScyllaResult<CreateAppResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.createApp({
        organizationId: wrapId(organizationId),
        name,
      });
      return response;
    }, 'Failed to create app.');
  }

  public deleteApp(appId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._appClient.deleteApp({ appId: wrapId(appId) });
    }, 'Failed to delete app.');
  }

  public setAppActive(
    appId: string,
    isActive: boolean,
  ): Promise<ScyllaResult<SetAppActiveResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.setAppActive({
        appId: wrapId(appId),
        isActive,
      });
      return response;
    }, 'Failed to update app.');
  }

  public listAppSecrets(appId: string): Promise<ScyllaResult<ListAppSecretsResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.listAppSecrets({ appId: wrapId(appId) });
      return response;
    }, 'Failed to list app secrets.');
  }

  public createAppSecret(
    appId: string,
    label: string,
  ): Promise<ScyllaResult<CreateAppSecretResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.createAppSecret({
        appId: wrapId(appId),
        label,
      });
      return response;
    }, 'Failed to create app secret.');
  }

  public revokeAppSecret(appSecretId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._appClient.revokeAppSecret({ appSecretId: wrapId(appSecretId) });
    }, 'Failed to revoke app secret.');
  }

  public setAppSecretEnabled(
    appSecretId: string,
    enabled: boolean,
  ): Promise<ScyllaResult<SetAppSecretEnabledResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._appClient.setAppSecretEnabled({
        appSecretId: wrapId(appSecretId),
        enabled,
      });
      return response;
    }, 'Failed to update app secret.');
  }
}

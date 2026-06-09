import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  App,
  AppSecret,
  CreatedApp,
  CreatedAppSecret,
} from '@/modules/features/apps/domain/models/app.model.ts';

/**
 * Repository interface for Apps. Apps are organization-scoped.
 */
export interface AppsRepository {
  listApps(organizationId: string): Promise<ScyllaResult<App[]>>;
  getApp(appId: string): Promise<ScyllaResult<App>>;
  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>>;
  deleteApp(appId: string): Promise<ScyllaResult<boolean>>;
  setAppActive(appId: string, active: boolean): Promise<ScyllaResult<App>>;

  listAppSecrets(appId: string): Promise<ScyllaResult<AppSecret[]>>;
  createAppSecret(appId: string, label: string): Promise<ScyllaResult<CreatedAppSecret>>;
  revokeAppSecret(secretId: string): Promise<ScyllaResult<boolean>>;
  setAppSecretEnabled(secretId: string, enabled: boolean): Promise<ScyllaResult<AppSecret>>;
}

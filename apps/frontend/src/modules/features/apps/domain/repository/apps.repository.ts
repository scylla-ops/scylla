import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  AppEntity,
  AppSecretEntity,
} from '@/modules/features/apps/domain/entities/app.entity.ts';
import type {
  CreatedApp,
  CreatedAppSecret,
} from '@/modules/features/apps/domain/structs/app.struct.ts';

/**
 * Repository interface for Apps. Apps are organization-scoped.
 */
export interface AppsRepository {
  listApps(organizationId: string): Promise<ScyllaResult<AppEntity[]>>;
  getApp(appId: string): Promise<ScyllaResult<AppEntity>>;
  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>>;
  deleteApp(appId: string): Promise<ScyllaResult<void>>;
  setAppActive(appId: string, active: boolean): Promise<ScyllaResult<AppEntity>>;

  listAppSecrets(appId: string): Promise<ScyllaResult<AppSecretEntity[]>>;
  createAppSecret(appId: string, label: string): Promise<ScyllaResult<CreatedAppSecret>>;
  revokeAppSecret(secretId: string): Promise<ScyllaResult<void>>;
  setAppSecretEnabled(secretId: string, enabled: boolean): Promise<ScyllaResult<AppSecretEntity>>;
}

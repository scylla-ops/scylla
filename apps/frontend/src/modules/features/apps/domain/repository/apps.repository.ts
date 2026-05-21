import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { App, CreatedApp } from '@/modules/features/apps/domain/models/app.model.ts';

/**
 * Repository interface for Apps. Apps are organization-scoped.
 */
export interface AppsRepository {
  listApps(organizationId: string): Promise<ScyllaResult<App[]>>;
  getApp(appId: string): Promise<ScyllaResult<App>>;
  createApp(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>>;
  deleteApp(appId: string): Promise<ScyllaResult<boolean>>;
}

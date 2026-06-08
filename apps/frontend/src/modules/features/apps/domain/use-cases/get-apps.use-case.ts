import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { App } from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: list the apps of an organization. */
export class GetAppsUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(organizationId: string): Promise<ScyllaResult<App[]>> {
    return this.repository.listApps(organizationId);
  }
}

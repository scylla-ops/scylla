import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: list the apps of an organization. */
export class GetAppsUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(organizationId: string): Promise<ScyllaResult<AppEntity[]>> {
    return this.repository.listApps(organizationId);
  }
}

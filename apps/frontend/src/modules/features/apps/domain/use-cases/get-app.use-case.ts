import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: fetch a single app by id. */
export class GetAppUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string): Promise<ScyllaResult<AppEntity>> {
    return this.repository.getApp(appId);
  }
}

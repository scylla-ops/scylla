import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { App } from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: fetch a single app by id. */
export class GetAppUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string): Promise<ScyllaResult<App>> {
    return this.repository.getApp(appId);
  }
}

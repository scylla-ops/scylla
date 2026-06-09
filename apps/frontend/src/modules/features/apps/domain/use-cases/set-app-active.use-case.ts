import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { App } from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: enable/disable a whole app. Disabling cuts all its tokens. */
export class SetAppActiveUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string, active: boolean): Promise<ScyllaResult<App>> {
    return this.repository.setAppActive(appId, active);
  }
}

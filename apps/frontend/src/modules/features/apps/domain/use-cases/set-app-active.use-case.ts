import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: enable/disable a whole app. Disabling cuts all its tokens. */
export class SetAppActiveUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string, active: boolean): Promise<ScyllaResult<AppEntity>> {
    return this.repository.setAppActive(appId, active);
  }
}

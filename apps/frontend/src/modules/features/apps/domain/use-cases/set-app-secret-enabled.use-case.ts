import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppSecretEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: enable/disable an app secret. Disabling cuts its tokens at once. */
export class SetAppSecretEnabledUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(secretId: string, enabled: boolean): Promise<ScyllaResult<AppSecretEntity>> {
    return this.repository.setAppSecretEnabled(secretId, enabled);
  }
}

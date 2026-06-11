import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppSecret } from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: list an app's secrets (metadata only). */
export class ListAppSecretsUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string): Promise<ScyllaResult<AppSecret[]>> {
    return this.repository.listAppSecrets(appId);
  }
}

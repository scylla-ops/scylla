import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CreatedAppSecret } from '@/modules/features/apps/domain/models/app.model.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: create/regenerate an app secret. Returns the one-time plaintext. */
export class CreateAppSecretUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string, label: string): Promise<ScyllaResult<CreatedAppSecret>> {
    return this.repository.createAppSecret(appId, label);
  }
}

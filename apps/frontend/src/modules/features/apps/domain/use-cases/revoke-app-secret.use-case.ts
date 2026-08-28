import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: permanently revoke (delete) an app secret. Cuts its tokens. */
export class RevokeAppSecretUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(secretId: string): Promise<ScyllaResult<void>> {
    return this.repository.revokeAppSecret(secretId);
  }
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { SecretRepository } from '@/modules/features/secret/domain/repository/secret.repository.ts';

/** Use case: delete a project-scoped secret. */
export class DeleteSecretUseCase {
  constructor(private readonly repository: SecretRepository) {}

  public execute(secretId: string): Promise<ScyllaResult<void>> {
    return this.repository.deleteById(secretId);
  }
}

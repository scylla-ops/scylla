import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Secret } from '@/modules/features/secret/domain/models/secret.model.ts';
import type { SecretRepository } from '@/modules/features/secret/domain/repository/secret.repository.ts';

/** Use case: list a project's secrets (metadata only). */
export class ListSecretsUseCase {
  constructor(private readonly repository: SecretRepository) {}

  public execute(projectId: string): Promise<ScyllaResult<Secret[]>> {
    return this.repository.listByProjectId(projectId);
  }
}

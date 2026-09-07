import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';
import type { SecretRepository } from '@/modules/features/secret/domain/repository/secret.repository.ts';

/** Use case: list a project's secrets (metadata only). */
export class ListSecretsUseCase {
  constructor(private readonly repository: SecretRepository) {}

  public execute(projectId: string): Promise<ScyllaResult<SecretEntity[]>> {
    return this.repository.listByProjectId(projectId);
  }
}

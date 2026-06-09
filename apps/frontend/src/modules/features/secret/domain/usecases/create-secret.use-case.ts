import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateSecretInput,
  SecretEntity,
} from '@/modules/features/secret/domain/entities/secret.entity.ts';
import type { SecretRepository } from '@/modules/features/secret/domain/repository/secret.repository.ts';

/** Use case: create a project-scoped secret. The value is write-only. */
export class CreateSecretUseCase {
  constructor(private readonly repository: SecretRepository) {}

  public execute(input: CreateSecretInput): Promise<ScyllaResult<SecretEntity>> {
    return this.repository.create(input);
  }
}

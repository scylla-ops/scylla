import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateSecretInput,
  SecretEntity,
} from '@/modules/features/secret/domain/entities/secret.entity.ts';
import type { SecretRepository } from '@/modules/features/secret/domain/repository/secret.repository.ts';
import type { SecretRemoteDataSource } from '@/modules/features/secret/infrastructure/repository/data-sources/secret-remote.data-source.ts';

/** SecretRepository backed by the remote (gRPC) data source. */
export class DefaultSecretRepository implements SecretRepository {
  constructor(private readonly remoteDataSource: SecretRemoteDataSource) {}

  public listByProjectId(projectId: string): Promise<ScyllaResult<SecretEntity[]>> {
    return this.remoteDataSource.listByProjectId(projectId);
  }

  public create(input: CreateSecretInput): Promise<ScyllaResult<SecretEntity>> {
    return this.remoteDataSource.create(input);
  }

  public deleteById(secretId: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.deleteById(secretId);
  }
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateSecretInput,
  SecretEntity,
} from '@/modules/features/secret/domain/entities/secret.entity.ts';

export interface SecretRemoteDataSource {
  listByProjectId(projectId: string): Promise<ScyllaResult<SecretEntity[]>>;
  create(input: CreateSecretInput): Promise<ScyllaResult<SecretEntity>>;
  deleteById(secretId: string): Promise<ScyllaResult<boolean>>;
}

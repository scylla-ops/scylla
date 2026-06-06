import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateSecretInput,
  Secret,
} from '@/modules/features/secret/domain/models/secret.model.ts';

/** Repository interface for project-scoped secrets. */
export interface SecretRepository {
  listByProjectId(projectId: string): Promise<ScyllaResult<Secret[]>>;
  create(input: CreateSecretInput): Promise<ScyllaResult<Secret>>;
  deleteById(secretId: string): Promise<ScyllaResult<boolean>>;
}

import type { AuthzVocabularyEntity } from '@/modules/features/permission/domain/entities/authz-vocabulary.entity.ts';
import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class ListAuthzVocabularyUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(): Promise<ScyllaResult<AuthzVocabularyEntity>> {
    return this._repository.listAuthzVocabulary();
  }
}

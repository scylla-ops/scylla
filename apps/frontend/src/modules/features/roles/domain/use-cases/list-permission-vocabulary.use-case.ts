import type { PermissionVocabularyEntity } from '@/modules/features/permission/domain/entities/permission-vocabulary.entity.ts';
import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class ListPermissionVocabularyUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(): Promise<ScyllaResult<PermissionVocabularyEntity>> {
    return this._repository.listPermissionVocabulary();
  }
}

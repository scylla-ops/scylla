import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class RevokeGrantUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(id: string): Promise<ScyllaResult<boolean>> {
    return this._repository.revokeGrant(id);
  }
}

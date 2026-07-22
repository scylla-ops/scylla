import type { PermissionScope } from '@/modules/features/role/domain/structs/permission.struct.ts';
import type { GrantEntity } from '@/modules/features/role/domain/entities/grant.entity.ts';
import type { PermissionRepository } from '@/modules/features/role/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class ListGrantsUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(scope?: PermissionScope, scopeId?: string): Promise<ScyllaResult<GrantEntity[]>> {
    return this._repository.listGrants(scope, scopeId);
  }
}

import type { PermissionScope } from '@/modules/features/role/domain/structs/permission.struct.ts';
import type { GrantableRoleEntity } from '@/modules/features/role/domain/entities/grantable-role.entity.ts';
import type { PermissionRepository } from '@/modules/features/role/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class ListGrantableRolesUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(scope?: PermissionScope): Promise<ScyllaResult<GrantableRoleEntity[]>> {
    return this._repository.listGrantableRoles(scope);
  }
}

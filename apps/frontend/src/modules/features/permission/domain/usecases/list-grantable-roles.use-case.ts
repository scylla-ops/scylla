import type { PermissionScope } from '@/modules/features/permission/domain/models/permission.model.ts';
import type { GrantableRoleEntity } from '@/modules/features/permission/domain/entities/grantable-role.entity.ts';
import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class ListGrantableRolesUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(scope?: PermissionScope): Promise<ScyllaResult<GrantableRoleEntity[]>> {
    return this._repository.listGrantableRoles(scope);
  }
}

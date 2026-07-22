import type { PermissionRepository } from '@/modules/features/role/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';

export class ListRolesUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(): Promise<ScyllaResult<RoleEntity[]>> {
    return this._repository.listRoles();
  }
}

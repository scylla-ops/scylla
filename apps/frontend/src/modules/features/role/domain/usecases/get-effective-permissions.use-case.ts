import type { PrincipalEntity } from '@/modules/features/role/domain/structs/permission.struct.ts';
import type { EffectivePermissionsEntity } from '@/modules/features/role/domain/entities/effective-permissions.entity.ts';
import type { PermissionRepository } from '@/modules/features/role/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class GetEffectivePermissionsUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(principal: PrincipalEntity): Promise<ScyllaResult<EffectivePermissionsEntity>> {
    return this._repository.getEffectivePermissions(principal);
  }
}

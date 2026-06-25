import type { PrincipalKind } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { EffectivePermissionsEntity } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class GetEffectivePermissionsUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(
    principalKind: PrincipalKind,
    principalId: string,
  ): Promise<ScyllaResult<EffectivePermissionsEntity>> {
    return this._repository.getEffectivePermissions(principalKind, principalId);
  }
}

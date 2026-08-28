import type { EffectivePermissionsEntity } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

/**
 * The signed-in caller's own access. Takes no principal: the server derives it
 * from the session, so asking for someone else's is not expressible here. Use
 * {@link GetEffectivePermissionsUseCase} for the admin view over others.
 */
export class GetMyPermissionsUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(): Promise<ScyllaResult<EffectivePermissionsEntity>> {
    return this._repository.getMyPermissions();
  }
}

import type { PermissionRepository } from '@/modules/features/role/domain/repository/permission.repository.ts';
import {
  type RoleEntity,
  updateRole,
} from '@/modules/features/role/domain/entities/role.entity.ts';
import { type ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AccessSpec } from '@/modules/features/role/domain/structs/permission.struct.ts';

export interface UpdateRoleInput {
  id: string;
  name?: string;
  description?: string;
  access?: AccessSpec;
}

export class UpdateRoleUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public async execute(input: UpdateRoleInput): Promise<ScyllaResult<RoleEntity>> {
    const result = await this._repository.getRole(input.id);

    return result
      .map(role => updateRole(role, input as Partial<RoleEntity>))
      .flatMapAsync(updatedRole => this._repository.updateRole(updatedRole));
  }
}

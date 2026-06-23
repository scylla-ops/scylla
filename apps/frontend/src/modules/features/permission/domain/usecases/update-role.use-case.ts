import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import {
  type RoleEntity,
  updateRole,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import { type ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';

export interface UpdateRoleInput {
  id: string;
  name?: string;
  description?: string;
  permissions?: Permission[];
  fullControl?: boolean;
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

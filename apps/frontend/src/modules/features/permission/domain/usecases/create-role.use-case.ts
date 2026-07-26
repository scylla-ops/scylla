import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  type AccessSpec,
  type PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface CreateRoleInput {
  name: string;
  description?: string;
  scope: PermissionScope;
  access: AccessSpec;
}

export class CreateRoleUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(input: CreateRoleInput): Promise<ScyllaResult<RoleEntity>> {
    const role: RoleCreationData = {
      name: input.name,
      description: input.description || '',
      scope: input.scope,
      access: input.access,
    };

    return this._repository.createRole(role);
  }
}

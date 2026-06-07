import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type {
  RoleCreationData,
  RoleEntity,
} from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  type Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/models/permission.model.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface CreateRoleInput {
  name: string;
  description?: string;
  permissions: Permission[];
  fullControl: boolean;
  scope: PermissionScope;
}

export class CreateRoleUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(input: CreateRoleInput): Promise<ScyllaResult<RoleEntity>> {
    const role: RoleCreationData = {
      name: input.name,
      description: input.description || '',
      permissions: input.permissions,
      fullControl: input.fullControl,
      scope: input.scope,
    };

    return this._repository.createRole(role);
  }
}

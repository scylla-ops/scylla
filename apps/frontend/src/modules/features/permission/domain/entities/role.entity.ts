import type { PermissionScope } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { Permission } from '@/generated/permission.ts';

export interface RoleEntity {
  id: string;
  /**
   * Stable key, set only for builtin roles (e.g. "organization-admin").
   */
  key?: string;
  name: string;
  description: string;

  /**
   * The scope kind a grant of this role must bind to. todo: see if necessary in the entity
   */
  scope: PermissionScope;

  builtin: boolean;
  /**
   * Full control: the role confers every permission within the grant's scope.
   * When true, `permissions` is empty.
   */
  fullControl: boolean;
  /**
   * The role's explicit permission set (when not full_control).
   */
  permissions: Permission[];
}

export interface RoleCreationData {
  name: string;
  description: string;
  permissions: Permission[];
  fullControl: boolean;
  scope: PermissionScope;
}

export const updateRole = (role: RoleEntity, changes: Partial<RoleEntity>): RoleEntity => {
  if (changes.name !== undefined && changes.name.trim() === '') {
    throw new Error('Le nom du rôle ne peut pas être vide');
  }

  return {
    ...role,
    ...changes,
    id: role.id,
  };
};

import type {
  AccessEntity,
  AccessSpec,
  PermissionScope,
} from '@/modules/features/role/domain/structs/permission.struct.ts';

/**
 * Where a role comes from. `unknown` means the backend sent an origin arm newer
 * than this build — never read it as "custom".
 */
export type RoleOrigin =
  | { kind: 'builtin'; key: string }
  | { kind: 'custom'; ownerOrganizationId?: string }
  | { kind: 'unknown' };

export interface RoleEntity {
  readonly id: string;
  readonly name: string;
  readonly description: string;

  /**
   * The scope kind a grant of this role must bind to. todo: see if necessary in the entity
   */
  scope: PermissionScope;

  /** Builtin (carries its stable key, e.g. "organization-admin") or custom. */
  origin: RoleOrigin;

  /** Full control over the scope, or an explicit permission set. */
  access: AccessEntity;
}

export interface RoleCreationData {
  name: string;
  description: string;
  scope: PermissionScope;
  access: AccessSpec;
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

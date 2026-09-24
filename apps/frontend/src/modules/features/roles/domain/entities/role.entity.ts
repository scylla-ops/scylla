import type { AccessEntity, AccessSpec, Permission, PermissionScope, } from '@platform/authz';

/** `unknown`: an origin arm newer than this build. Never read it as "custom". */
export type RoleOrigin =
  | { kind: 'builtin'; key: string }
  | { kind: 'custom'; ownerOrganizationId?: string }
  | { kind: 'unknown' };

export interface RoleEntity {
  readonly id: string;
  readonly name: string;
  readonly description: string;

  /** The scope kind a grant of this role binds to. */
  scope: PermissionScope;

  origin: RoleOrigin;

  access: AccessEntity;
}

export interface RoleCreationData {
  name: string;
  description: string;
  scope: PermissionScope;
  access: AccessSpec;
}

/**
 * An unknown access arm, or a role missing from the catalog, counts as conferring,
 * unlike `canAccess`: here the question is about someone else's role, and hiding
 * access that exists would mislead an administrator.
 */
export const roleConfers = (role: RoleEntity | undefined, permission: Permission): boolean => {
  if (!role) return true;
  switch (role.access.kind) {
    case 'fullControl':
      return true;
    case 'restricted':
      return role.access.permissions.includes(permission);
    default:
      return true;
  }
};

export const updateRole = (role: RoleEntity, changes: Partial<RoleEntity>): RoleEntity => {
  if (changes.name !== undefined && changes.name.trim() === '') {
    throw new Error('Role name cannot be empty');
  }

  return {
    ...role,
    ...changes,
    id: role.id,
  };
};

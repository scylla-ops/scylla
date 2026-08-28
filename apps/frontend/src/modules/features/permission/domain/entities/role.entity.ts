import type {
  AccessEntity,
  AccessSpec,
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

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

/**
 * Whether a role confers `permission`.
 *
 * An access arm this build cannot read — or a role missing from the catalog —
 * counts as conferring. That is the opposite of `canAccess`, deliberately:
 * there, denying on the unknown protects the UI from showing what the user may
 * not hold; here, the question is asked *about someone else's* role, so denying
 * would hide access that exists and let an administrator act on a list they
 * cannot trust. Reading the catalog is itself a permission not every
 * administrator holds, so `undefined` is a routine answer, not an anomaly.
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
    throw new Error('Le nom du rôle ne peut pas être vide');
  }

  return {
    ...role,
    ...changes,
    id: role.id,
  };
};

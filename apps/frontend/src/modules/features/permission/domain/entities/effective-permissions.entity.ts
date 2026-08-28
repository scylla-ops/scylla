import {
  Permission,
  PermissionScope,
  type AccessEntity,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * The permissions a principal holds at one scope (roles expanded +
 * direct permission grants).
 */
export interface EffectiveScopeEntity {
  scope: PermissionScope;
  /** The org/project id the grants are bound to; empty for SYSTEM scope. */
  scopeId: string;
  /** Full control over this scope, or an explicit permission set. */
  access: AccessEntity;
}

/** All scopes at which a principal currently holds permissions. */
export interface EffectivePermissionsEntity {
  scopes: EffectiveScopeEntity[];
}

/**
 * The resource an authorization check is about. A permission granted at a
 * broader scope covers narrower targets within it: a SYSTEM grant covers every
 * organization and project, an ORGANIZATION grant covers that org and all its
 * projects, a PROJECT grant covers only that project.
 */
export interface PermissionTarget {
  organizationId?: string;
  projectId?: string;
}

/**
 * Permissions that another one carries with it, keyed by the permission being
 * asked about. Holding any of the listed permissions is holding this one.
 *
 * Only one entry, and it is a rule rather than a shortcut: editing the role
 * catalog and handing roles out are a single capability — see
 * `presentation/utils/permission-mapping.ts`, where ticking "manage roles"
 * writes `MANAGE_SYSTEM_GRANTS` too. This mirrors that on the read side so a
 * role saved before the rule existed still opens the doors the rule promises.
 */
const IMPLIED_BY: Partial<Record<Permission, Permission[]>> = {
  [Permission.MANAGE_SYSTEM_GRANTS]: [Permission.MANAGE_ROLES],
};

/** Whether an access arm confers `permission`. `unknown` never confers. */
const accessConfers = (access: AccessEntity, permission: Permission): boolean => {
  switch (access.kind) {
    case 'fullControl':
      return true;
    case 'restricted':
      return (
        access.permissions.includes(permission) ||
        (IMPLIED_BY[permission] ?? []).some(implier => access.permissions.includes(implier))
      );
    default:
      return false;
  }
};

/**
 * Whether a principal with these effective permissions may perform `permission`
 * on `target`. True as soon as any effective scope both confers the permission
 * and covers the target:
 *  - SYSTEM       covers everything;
 *  - ORGANIZATION covers the target's organization (and, by extension, its projects);
 *  - PROJECT      covers the target's project.
 *
 * Pure — safe to call anywhere. Client-side UX only; the backend remains the
 * source of truth for enforcement.
 */
export const canAccess = (
  effective: EffectivePermissionsEntity,
  permission: Permission,
  target: PermissionTarget = {},
): boolean =>
  effective.scopes.some(entry => {
    if (!accessConfers(entry.access, permission)) return false;
    switch (entry.scope) {
      case PermissionScope.SYSTEM:
        return true;
      case PermissionScope.ORGANIZATION:
        return !!target.organizationId && entry.scopeId === target.organizationId;
      case PermissionScope.PROJECT:
        return !!target.projectId && entry.scopeId === target.projectId;
      default:
        return false;
    }
  });

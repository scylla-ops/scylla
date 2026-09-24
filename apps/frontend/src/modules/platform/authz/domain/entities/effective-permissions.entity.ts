import {
  Permission,
  PermissionScope,
  type AccessEntity,
} from '@platform/authz/domain/structs/permission.struct.ts';

/** The permissions held at one scope: roles expanded, plus direct grants. */
export interface EffectiveScopeEntity {
  scope: PermissionScope;
  /** Empty for the SYSTEM scope. */
  scopeId: string;
  access: AccessEntity;
}

export interface EffectivePermissionsEntity {
  scopes: EffectiveScopeEntity[];
}

/**
 * What a check is about. A grant covers narrower targets: SYSTEM covers
 * everything, ORGANIZATION covers the organization and its projects.
 */
export interface PermissionTarget {
  organizationId?: string;
  projectId?: string;
}

/**
 * Holding a listed permission is holding the key. Managing roles implies
 * managing system grants (see `permission-mapping.ts`), also for older roles.
 */
const IMPLIED_BY: Partial<Record<Permission, Permission[]>> = {
  [Permission.MANAGE_SYSTEM_GRANTS]: [Permission.MANAGE_ROLES],
};

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

/** Client-side UX only: the backend enforces. */
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

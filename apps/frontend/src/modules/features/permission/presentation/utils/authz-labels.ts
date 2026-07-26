import {
  Permission,
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

// ── Scope ─────────────────────────────────────────────────────────────────────

export const ALL_SCOPES: PermissionScope[] = [
  PermissionScope.SYSTEM,
  PermissionScope.ORGANIZATION,
  PermissionScope.PROJECT,
];

export const scopeName = (scope: PermissionScope): string => {
  switch (scope) {
    case PermissionScope.SYSTEM:
      return 'System';
    case PermissionScope.ORGANIZATION:
      return 'Organization';
    case PermissionScope.PROJECT:
      return 'Project';
    default:
      return 'Unknown';
  }
};

// ── Permission ────────────────────────────────────────────────────────────────

export const ALL_PERMISSIONS: Permission[] = Object.values(Permission).filter(
  (v): v is Permission => typeof v === 'number' && v !== Permission.UNSPECIFIED,
);

export const permissionName = (permission: Permission): string =>
  Permission[permission]?.replace(/_/g, ' ').toLowerCase() ?? String(permission);

// ── Principal ─────────────────────────────────────────────────────────────────

export const principalKindName = (kind: PrincipalKind): string => {
  switch (kind) {
    case PrincipalKind.USER:
      return 'User';
    case PrincipalKind.APP:
      return 'App';
    default:
      return 'Unknown';
  }
};

// ── Errors ────────────────────────────────────────────────────────────────────

export const errorMessage = (error: unknown): string => {
  if (error == null) return 'Unknown error';
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return JSON.stringify(error);
};

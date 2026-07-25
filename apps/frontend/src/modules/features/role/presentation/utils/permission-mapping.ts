import {
  Permission,
  PermissionScope,
} from '@/modules/features/role/domain/structs/permission.struct.ts';

export const PERMISSIONS_BY_SCOPE: Map<PermissionScope, Permission[]> = new Map([
  [
    PermissionScope.SYSTEM,
    [
      Permission.CREATE_AGENT,
      Permission.CREATE_ORGANIZATION,
      Permission.DELETE_ORGANIZATION,
      Permission.CREATE_PROJECT,
      Permission.UPDATE_PROJECT,
      Permission.DELETE_PROJECT,
      Permission.UPDATE_ORGANIZATION,
      Permission.ADD_ORGANIZATION_MEMBER,
      Permission.REMOVE_ORGANIZATION_MEMBER,
      Permission.DELETE_USER,
      Permission.CREATE_USER,
      Permission.MANAGE_ROLES,
      Permission.MANAGE_ORG_GRANTS,
      Permission.MANAGE_PROJECT_GRANTS,
    ],
  ],
  [
    PermissionScope.ORGANIZATION,
    [
      Permission.UPDATE_ORGANIZATION,
      Permission.DELETE_ORGANIZATION,
      Permission.ADD_ORGANIZATION_MEMBER,
      Permission.REMOVE_ORGANIZATION_MEMBER,
      Permission.MANAGE_ORG_GRANTS,
      Permission.CREATE_PROJECT,
      Permission.UPDATE_PROJECT,
      Permission.DELETE_PROJECT,
      Permission.MANAGE_PROJECT_GRANTS,
    ],
  ],
]);

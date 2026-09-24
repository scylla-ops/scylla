import type { PermissionScope, RoleKind, } from '@platform/authz';

/** A role that can be granted: identity and scope only, for a picker. */
export interface GrantableRoleEntity {
  roleId: string;
  scope: PermissionScope;
  kind: RoleKind;
  description: string;
}

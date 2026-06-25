import type { PermissionScope } from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * A role that may be assigned via a grant.
 * Lighter than {@link RoleEntity}: no permission list, just the identity
 * and scope metadata needed to build a grant picker.
 */
export interface GrantableRoleEntity {
  /** Role id or stable key (e.g. "organization-admin"). */
  name: string;
  scope: PermissionScope;
  /** "admin" (full control) | "agent" (restricted) */
  kind: string;
  description: string;
}

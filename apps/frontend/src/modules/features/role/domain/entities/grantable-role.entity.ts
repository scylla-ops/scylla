import type {
  PermissionScope,
  RoleKind,
} from '@/modules/features/role/domain/structs/permission.struct.ts';

/**
 * A role that may be assigned via a grant.
 * Lighter than {@link RoleEntity}: no permission list, just the identity
 * and scope metadata needed to build a grant picker.
 */
export interface GrantableRoleEntity {
  /** Role id (value of the protobuf RoleId wrapper). */
  roleId: string;
  scope: PermissionScope;
  /** ADMIN (full control) | AGENT (restricted). */
  kind: RoleKind;
  description: string;
}

import type {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/models/permission.model.ts';

export interface GrantEntity {
  id: string;
  /** String value extracted from the protobuf UserId wrapper. */
  userId?: string;
  /**
   * Role id or key being granted (e.g. "organization-admin").
   * Empty string when this is a direct single-permission grant.
   */
  role: string;
  scope: PermissionScope;
  /** The org/project id the grant is bound to; empty for SYSTEM scope. */
  scopeId: string;
  /** Set instead of `role` when this is a direct single-permission grant. */
  permission?: Permission;
}

import type {
  PermissionScope,
  PrincipalEntity,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

export interface GrantEntity {
  id: string;
  /** Who holds the grant: a user or an app. */
  principal: PrincipalEntity;
  /**
   * The role it confers within its scope. A grant carries a role and nothing
   * else; anything narrower is a role with exactly the permissions wanted.
   */
  roleId: string;
  scope: PermissionScope;
  /** The org/project id the grant is bound to; empty for SYSTEM scope. */
  scopeId: string;
}

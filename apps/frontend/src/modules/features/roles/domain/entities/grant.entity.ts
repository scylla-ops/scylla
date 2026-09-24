import type { PermissionScope, PrincipalEntity, } from '@platform/authz';

export interface GrantEntity {
  id: string;
  principal: PrincipalEntity;
  /** A grant carries a role and nothing else: a narrower grant is a role with fewer permissions. */
  roleId: string;
  scope: PermissionScope;
  scopeId: string;
}

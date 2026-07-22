import type {
  AccessEntity,
  PermissionScope,
} from '@/modules/features/role/domain/structs/permission.struct.ts';

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

import type {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * The permissions a principal holds at one scope (roles expanded +
 * direct permission grants).
 */
export interface EffectiveScopeEntity {
  scope: PermissionScope;
  /** The org/project id the grants are bound to; empty for SYSTEM scope. */
  scopeId: string;
  /**
   * Full control: the principal holds every permission within this scope.
   * When true, `permissions` is empty.
   */
  fullControl: boolean;
  permissions: Permission[];
}

/** All scopes at which a principal currently holds permissions. */
export interface EffectivePermissionsEntity {
  scopes: EffectiveScopeEntity[];
}

import type {
  Permission,
  PermissionScope,
} from '@/modules/features/role/domain/structs/permission.struct.ts';

/**
 * One entry of the authz vocabulary: a permission and the narrowest scope
 * at which it is coherent (i.e. its target resource exists).
 */
export interface AuthzActionEntity {
  permission: Permission;
  /**
   * The minimal (narrowest) scope a role may bind to and still have this
   * permission do something. Usable at `minScope` or any broader ancestor scope.
   *
   * `PermissionScope` values are ordered broad→narrow
   * (SYSTEM=1 < ORGANIZATION=2 < PROJECT=3), so "broader or equal" is
   * simply `scope <= minScope`.
   */
  minScope: PermissionScope;
}

/** The full authz vocabulary published by the backend. */
export interface AuthzVocabularyEntity {
  actions: AuthzActionEntity[];
}

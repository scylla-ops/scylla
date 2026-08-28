import { useCallback } from 'react';
import { useLingui } from '@lingui/react/macro';
import type {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import {
  getPermissionDefinition,
  humanizePermission,
  SCOPE_LABELS,
} from '@/modules/features/permission/presentation/utils/permission-mapping.ts';

/**
 * Display names for permissions and scopes. Catalog entries are translated;
 * a permission outside the catalog — one the backend or another client put on
 * the role — falls back to its humanized enum key rather than disappearing
 * behind a generic "unknown".
 */
export const usePermissionLabels = () => {
  const { i18n } = useLingui();

  /**
   * `roleScope` is the scope of the role carrying the permission, not the
   * permission's own. Pass it wherever the reader is looking at one specific
   * role: a project permission in an organization role applies to every project
   * of that organization, and gets the plural wording that says so.
   */
  const permissionLabel = useCallback(
    (permission: Permission, roleScope?: PermissionScope): string => {
      const definition = getPermissionDefinition(permission);
      if (!definition) return humanizePermission(permission);
      const broadened = roleScope !== undefined && roleScope !== definition.scope;
      return i18n._(broadened ? (definition.broadLabel ?? definition.label) : definition.label);
    },
    [i18n],
  );

  const scopeLabel = useCallback((scope: PermissionScope): string => i18n._(SCOPE_LABELS[scope]), [
    i18n,
  ]);

  return { permissionLabel, scopeLabel };
};

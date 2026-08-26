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

  const permissionLabel = useCallback(
    (permission: Permission): string => {
      const definition = getPermissionDefinition(permission);
      return definition ? i18n._(definition.label) : humanizePermission(permission);
    },
    [i18n],
  );

  const scopeLabel = useCallback((scope: PermissionScope): string => i18n._(SCOPE_LABELS[scope]), [
    i18n,
  ]);

  return { permissionLabel, scopeLabel };
};

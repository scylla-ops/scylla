import { useCallback, useMemo } from 'react';
import { PermissionScope } from '@platform/authz';
import { useOrganizations } from '@/modules/features/organization';
import { useProjectsByOrganizations } from '@/modules/features/project';

/** A grant's scope target, resolved to human-readable names. */
export interface GrantTargetLabel {
  /** Project/organization name, or the raw id while it is still resolving. */
  name: string;
  /** For project scope: the owning organization's name. */
  organizationName?: string;
  /** False while the name is still being fetched (falls back to the id). */
  resolved: boolean;
}

/**
 * Resolves a grant's `scopeId` to a display name for a given role scope.
 * - SYSTEM       → "System".
 * - ORGANIZATION → the organization name (from {@link useOrganizations}).
 * - PROJECT      → the project name (+ its org), fanned out across the user's
 *                  organizations since a project grant only carries the project id.
 */
export const useGrantTargetLabels = (scope: PermissionScope) => {
  const { organizations } = useOrganizations();

  const orgIds = useMemo(() => (organizations ?? []).map(org => org.id), [organizations]);

  const orgNameById = useMemo(
    () => new Map((organizations ?? []).map(org => [org.id, org.name])),
    [organizations],
  );

  // Only project scope needs the lookup — a project grant carries the project
  // id alone, so resolving it to a name means fanning out over the user's
  // organizations. `project` owns that query.
  const projectInfoById = useProjectsByOrganizations(
    orgIds,
    scope === PermissionScope.PROJECT,
  );

  const labelFor = useCallback(
    (scopeId: string): GrantTargetLabel => {
      if (scope === PermissionScope.SYSTEM || scopeId === '') {
        return { name: 'System', resolved: true };
      }
      if (scope === PermissionScope.ORGANIZATION) {
        const name = orgNameById.get(scopeId);
        return { name: name ?? scopeId, resolved: name !== undefined };
      }
      const info = projectInfoById.get(scopeId);
      if (!info) return { name: scopeId, resolved: false };
      return {
        name: info.name,
        organizationName: orgNameById.get(info.organizationId) ?? info.organizationId,
        resolved: true,
      };
    },
    [scope, orgNameById, projectInfoById],
  );

  return { labelFor };
};

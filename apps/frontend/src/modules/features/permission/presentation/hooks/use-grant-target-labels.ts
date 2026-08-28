import { useCallback, useMemo } from 'react';
import { useQueries } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import { PermissionScope } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';

/** Big page so a single request covers every project of an org for the lookup. */
const LOOKUP_PAGE_SIZE = 100;

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
  const { getProjects } = useDependencies().project;

  const orgIds = useMemo(
    () => (organizations ?? []).map(org => idValue(org.organizationId)),
    [organizations],
  );

  const orgNameById = useMemo(
    () => new Map((organizations ?? []).map(org => [idValue(org.organizationId), org.name])),
    [organizations],
  );

  // Only project scope needs the per-org project lists. `combine` folds the
  // fan-out into a single projectId → {name, orgId} lookup, memoized by
  // TanStack Query on the underlying results.
  const projectInfoById = useQueries({
    queries:
      scope === PermissionScope.PROJECT
        ? orgIds.map(orgId => ({
            queryKey: ['projects', orgId, { page: 1, pageSize: LOOKUP_PAGE_SIZE }] as const,
            queryFn: async () =>
              (await getProjects.execute(orgId, { page: 1, pageSize: LOOKUP_PAGE_SIZE })).unwrap(),
          }))
        : [],
    combine: results => {
      const map = new Map<string, { name: string; orgId: string }>();
      results.forEach((result, index) => {
        const orgId = orgIds[index];
        for (const project of result.data?.projects ?? []) {
          map.set(project.id, { name: project.name, orgId });
        }
      });
      return map;
    },
  });

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
        organizationName: orgNameById.get(info.orgId) ?? info.orgId,
        resolved: true,
      };
    },
    [scope, orgNameById, projectInfoById],
  );

  return { labelFor };
};

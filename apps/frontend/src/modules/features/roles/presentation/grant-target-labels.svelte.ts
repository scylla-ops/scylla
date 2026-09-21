import { PermissionScope } from '@platform/authz';
import { createQueries, createQuery } from '@platform/query';
import { organizationQueries } from '@/modules/features/organization';
import { projectLookupQueries } from '@/modules/features/project';

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
 *
 * - SYSTEM       → "System".
 * - ORGANIZATION → the organization name (from `organizationQueries.mine`).
 * - PROJECT      → the project name (+ its org), fanned out across the user's
 *                  organizations since a project grant only carries the project id.
 *
 * The fan-out is `createQueries` with the `combine` the `project` module owns:
 * one query per organization, folded into a single map there so the fold is
 * memoized on the results rather than rebuilt on every read.
 */
export const createGrantTargetLabels = (scope: () => PermissionScope) => {
  const organizationsQuery = createQuery(() => organizationQueries.mine());
  const organizations = $derived(organizationsQuery.data ?? []);

  // Rebuilt whole by the `$derived` and never mutated after it is read, so a
  // reactive collection would only make a throwaway object track dependencies.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const orgNameById = $derived(new Map(organizations.map(org => [org.id, org.name])));

  // Only project scope needs the lookup — a project grant carries the project id
  // alone, so resolving it to a name means fanning out over the organizations.
  const lookup = $derived(
    projectLookupQueries(
      organizations.map(org => org.id),
      scope() === PermissionScope.PROJECT,
    ),
  );

  const results = createQueries(() => ({ queries: lookup.queries }));

  /**
   * The fold is applied here rather than passed to `createQueries` as its
   * `combine`. TanStack's Svelte binding wraps a combined result in a raw ref
   * that re-exposes its **top-level keys** — which turns a `Map` into an empty
   * object, silently, with every label falling back to a raw id. An array of
   * results survives that wrapper, so the map is built from it in a `$derived`,
   * which recomputes on the same cadence anyway.
   */
  const projectInfoById = $derived(
    lookup.combine([...results] as { data?: { projects: { id: string; name: string }[] } }[]),
  );

  return {
    labelFor: (scopeId: string): GrantTargetLabel => {
      const current = scope();
      if (current === PermissionScope.SYSTEM || scopeId === '') {
        return { name: 'System', resolved: true };
      }
      if (current === PermissionScope.ORGANIZATION) {
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
  };
};

export type GrantTargetLabels = ReturnType<typeof createGrantTargetLabels>;

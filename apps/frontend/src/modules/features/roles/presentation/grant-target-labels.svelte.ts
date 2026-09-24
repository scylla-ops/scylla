import { PermissionScope } from '@platform/authz';
import { createQueries, createQuery } from '@platform/query';
import { organizationQueries } from '@/modules/features/organization';
import { projectLookupQueries } from '@/modules/features/project';

export interface GrantTargetLabel {
  name: string;
  organizationName?: string;
  resolved: boolean;
}

/**
 * Resolves a grant's `scopeId` to a name. A project grant carries only the project
 * id, so the projects of every organization of the user are fetched.
 */
export const createGrantTargetLabels = (scope: () => PermissionScope) => {
  const organizationsQuery = createQuery(() => organizationQueries.mine());
  const organizations = $derived(organizationsQuery.data ?? []);

  // Rebuilt whole by the `$derived` and never mutated after it is read, so a
  // reactive collection would only make a throwaway object track dependencies.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const orgNameById = $derived(new Map(organizations.map(org => [org.id, org.name])));

  const lookup = $derived(
    projectLookupQueries(
      organizations.map(org => org.id),
      scope() === PermissionScope.PROJECT,
    ),
  );

  const results = createQueries(() => ({ queries: lookup.queries }));

  /**
   * Folded here, not in the `combine` of `createQueries`: the Svelte binding turns a
   * combined `Map` into an empty object.
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

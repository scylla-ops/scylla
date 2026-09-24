import { SvelteMap } from 'svelte/reactivity';
import { PermissionScope, PrincipalKind } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { organizationQueries } from '@/modules/features/organization';
import { projectQueries } from '@/modules/features/project';
import { userQueries } from '@/modules/features/user';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import { buildGrantEligibility, type GrantEligibility } from './grant-eligibility.calculator.ts';
import { grantMutations, roleQueries } from './roles.queries.ts';

export interface TargetOption {
  id: string;
  name: string;
}

export interface UserOption {
  id: string;
  name: string;
  /** Absent when the user can receive the grant. */
  ineligible?: Exclude<GrantEligibility, 'eligible'>;
}

/**
 * Grants one role to one user, one grant per selected target. A project grant
 * needs the user admitted to the organization first: the users who are not
 * carry the reason (a value; the component words it).
 */
export const createGrantCreator = (role: () => RoleEntity) => {
  const scope = $derived(role().scope);
  const isProjectScope = $derived(scope === PermissionScope.PROJECT);
  const needsTargets = $derived(scope !== PermissionScope.SYSTEM);

  let userId = $state('');
  /** Id → name. Accumulates across organizations. */
  const selected = new SvelteMap<string, string>();
  let browseOrgId = $state<string | null>(null);

  const usersQuery = createQuery(() => userQueries.list());
  const organizationsQuery = createQuery(() => organizationQueries.mine());
  const projectsQuery = createQuery(() => projectQueries.byOrganization(browseOrgId));
  const grantsQuery = createQuery(() => roleQueries.allGrants());
  const rolesQuery = createQuery(() => roleQueries.catalog());

  const createGrant = createMutation(() => grantMutations.create());

  const eligibilityFor = $derived(
    buildGrantEligibility(grantsQuery.data ?? [], rolesQuery.data ?? [], browseOrgId),
  );

  /** Everyone stays listed; who cannot receive the grant carries the reason. */
  const users = $derived.by((): UserOption[] => {
    const all = (usersQuery.data?.items ?? []).map(user => ({
      id: user.userId,
      name: user.username,
    }));
    if (!isProjectScope || !browseOrgId) return all;

    return all.map(user => {
      const eligibility = eligibilityFor(user.id);
      return eligibility === 'eligible' ? user : { ...user, ineligible: eligibility };
    });
  });

  const hasSelectableUser = $derived(users.some(user => !user.ineligible));

  /** Scope ids where the user already holds this role: offered disabled. */
  const alreadyGranted = $derived.by(() => {
    // Rebuilt whole by the `$derived` and never mutated after it is read, so a
    // reactive collection would only make a throwaway object track dependencies.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const ids = new Set<string>();
    if (!userId) return ids;
    for (const grant of grantsQuery.data ?? []) {
      if (
        grant.roleId === role().id &&
        grant.principal.kind === PrincipalKind.USER &&
        grant.principal.id === userId
      ) {
        ids.add(grant.scopeId);
      }
    }
    return ids;
  });

  const organizations = $derived(
    (organizationsQuery.data ?? []).map(org => ({ id: org.id, name: org.name })),
  );

  const projects = $derived(
    (projectsQuery.data?.projects ?? []).map(project => ({
      id: project.id,
      name: project.name,
    })),
  );

  const reset = () => {
    userId = '';
    selected.clear();
    browseOrgId = null;
  };

  return {
    get scope() {
      return scope;
    },
    get isProjectScope() {
      return isProjectScope;
    },
    get needsTargets() {
      return needsTargets;
    },
    get userId() {
      return userId;
    },
    set userId(next: string) {
      userId = next;
    },
    get browseOrgId() {
      return browseOrgId;
    },
    get users() {
      return users;
    },
    get hasSelectableUser() {
      return hasSelectableUser;
    },
    get organizations() {
      return organizations;
    },
    get organizationsLoading() {
      return organizationsQuery.isLoading;
    },
    get projects() {
      return projects;
    },
    get projectsLoading() {
      return projectsQuery.isLoading;
    },
    get selected() {
      return [...selected.entries()].map(([id, name]) => ({ id, name }));
    },
    get selectedCount() {
      return selected.size;
    },
    get isPending() {
      return createGrant.isPending;
    },
    get isValid() {
      return userId !== '' && (!needsTargets || selected.size > 0);
    },
    isAlreadyGranted: (targetId: string) => alreadyGranted.has(targetId),
    isSelected: (targetId: string) => selected.has(targetId),
    toggle: (option: TargetOption) => {
      if (selected.has(option.id)) selected.delete(option.id);
      else selected.set(option.id, option.name);
    },
    /** Drops the picks that the new organization makes invalid. */
    browseOrganization: (organizationId: string) => {
      browseOrgId = organizationId;
      if (users.some(user => user.id === userId && user.ineligible)) userId = '';
    },
    reset,
    /** Returns how many grants were created, for the toast. The mutation cache toasts the failures. */
    submit: async (): Promise<number | null> => {
      const current = role();
      const scopeIds =
        current.scope === PermissionScope.SYSTEM ? [''] : [...selected.keys()];

      try {
        await Promise.all(
          scopeIds.map(scopeId =>
            createGrant.mutateAsync({
              principal: { kind: PrincipalKind.USER, id: userId },
              roleId: current.id,
              scope: current.scope,
              scopeId,
            }),
          ),
        );
        return scopeIds.length;
      } catch {
        return null;
      }
    },
  };
};

export type GrantCreator = ReturnType<typeof createGrantCreator>;

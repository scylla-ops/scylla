import { SvelteMap } from 'svelte/reactivity';
import { PermissionScope, PrincipalKind } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { organizationQueries } from '@/modules/features/organization';
import { projectQueries } from '@/modules/features/project';
import { userQueries } from '@/modules/features/user';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import { buildGrantEligibility, type GrantEligibility } from './grant-eligibility.calculator.ts';
import { grantMutations, roleQueries } from './roles.queries.ts';

/** A selectable grant target: the scope id (org/project) plus its display name. */
export interface TargetOption {
  id: string;
  name: string;
}

/** A user offered in the picker; `ineligible` greys them out and says why. */
export interface UserOption {
  id: string;
  name: string;
  /** Absent when the user can receive this grant. */
  ineligible?: Exclude<GrantEligibility, 'eligible'>;
}

/**
 * Granting one role to one user, across the scope targets the role requires.
 *
 * - SYSTEM       → the user, system-wide (no scope id).
 * - ORGANIZATION → the user, on one or more organizations.
 * - PROJECT      → the user, on one or more projects of a chosen organization.
 *
 * One grant is created per selected target.
 *
 * Project grants follow the backend's tenant boundary: a user may only receive
 * one once the organization owning the project has already admitted them — that
 * is what being a member of it means. So the dialog asks for the organization
 * first and marks the users it has not admitted, turning a server-side
 * rejection into a constraint you can see. **The reason is a value, not a
 * message**: which sentence explains it is the component's business.
 */
export const createGrantCreator = (role: () => RoleEntity) => {
  const scope = $derived(role().scope);
  const isProjectScope = $derived(scope === PermissionScope.PROJECT);
  /** Every scope but SYSTEM grants *somewhere*, so it needs targets picked. */
  const needsTargets = $derived(scope !== PermissionScope.SYSTEM);

  let userId = $state('');
  /** Chosen targets, id → display name; it accumulates across organizations. */
  const selected = new SvelteMap<string, string>();
  /** For PROJECT scope: whose projects are currently being browsed. */
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

  /**
   * Everyone stays in the list; those who cannot receive this grant carry the
   * reason, so the constraint is visible rather than a silently shorter list.
   */
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

  /** Scope ids where this user already holds this role — offered as disabled. */
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
    /**
     * Switching organization changes who is eligible, so a pick that is no
     * longer valid is dropped with it — the React version needed an effect
     * watching the recomputed list to notice.
     */
    browseOrganization: (organizationId: string) => {
      browseOrgId = organizationId;
      if (users.some(user => user.id === userId && user.ineligible)) userId = '';
    },
    reset,
    /**
     * Creates one grant per selected target, and answers how many landed so the
     * caller can word its toast. Throws nothing: the mutation cache has already
     * toasted the failure, and the dialog stays open on it.
     */
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

import { can, Permission } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import { roleMutations, roleQueries } from './roles.queries.ts';

/** Roles and every grant, fetched once for the list and the detail. The holder count comes from the grants. */
export const createRolesPage = () => {
  const rolesQuery = createQuery(() => roleQueries.catalog());
  const grantsQuery = createQuery(() => roleQueries.allGrants());
  const deleteRole = createMutation(() => roleMutations.remove());

  const roles = $derived(rolesQuery.data ?? []);
  const grants = $derived(grantsQuery.data ?? []);

  /** Also carries grant management: holding it opens this page. */
  const canManageRoles = $derived(can(Permission.MANAGE_ROLES));

  /** Builtin roles cannot be deleted: they stay out of the selection. */
  const deletableRoleIds = $derived(
    roles.filter(role => role.origin.kind === 'custom').map(role => role.id),
  );

  const selection = createFeatureSelection('roles', () => deletableRoleIds, {
    deleteItem: (id: string) => deleteRole.mutateAsync(id),
  });

  const memberCounts = $derived.by(() => {
    // Rebuilt whole by the `$derived` and never mutated after it is read, so a
    // reactive collection would only make a throwaway object track dependencies.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const counts = new Map<string, number>();
    for (const grant of grants) {
      counts.set(grant.roleId, (counts.get(grant.roleId) ?? 0) + 1);
    }
    return counts;
  });

  let activeRoleId = $state<string | null>(null);
  /** `null` when creating. */
  let editingRole = $state<RoleEntity | null>(null);
  let formOpen = $state(false);

  const activeRole = $derived(roles.find(role => role.id === activeRoleId) ?? null);

  return {
    get roles() {
      return roles;
    },
    get isLoading() {
      return rolesQuery.isLoading;
    },
    get canManageRoles() {
      return canManageRoles;
    },
    get activeRole() {
      return activeRole;
    },
    get activeRoleId() {
      return activeRoleId;
    },
    get formOpen() {
      return formOpen;
    },
    get editingRole() {
      return editingRole;
    },
    selection,
    memberCountOf: (roleId: string) => memberCounts.get(roleId) ?? 0,
    isSelectable: (role: RoleEntity) => canManageRoles && role.origin.kind === 'custom',
    open: (roleId: string) => {
      activeRoleId = roleId;
    },
    openCreate: () => {
      editingRole = null;
      formOpen = true;
    },
    openEdit: (role: RoleEntity) => {
      editingRole = role;
      formOpen = true;
    },
    closeForm: () => {
      formOpen = false;
    },
  };
};

export type RolesPage = ReturnType<typeof createRolesPage>;

import { can, Permission } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import { roleMutations, roleQueries } from './roles.queries.ts';

/**
 * The roles screen: the catalog on the left, one role's detail on the right.
 *
 * Both halves read the same two lists — the roles and *every* grant — so they
 * are fetched once here and handed down, rather than each panel asking for
 * itself and drawing two cache entries for one resource.
 *
 * Grants are also what answers "how many people hold this role", with no extra
 * request: the count is a fold over the list the detail panel needs anyway.
 */
export const createRolesPage = () => {
  const rolesQuery = createQuery(() => roleQueries.catalog());
  const grantsQuery = createQuery(() => roleQueries.allGrants());
  const deleteRole = createMutation(() => roleMutations.remove());

  const roles = $derived(rolesQuery.data ?? []);
  const grants = $derived(grantsQuery.data ?? []);

  /**
   * Editing the catalog is a system capability — and the one that carries grant
   * management with it, so holding it is what opens this whole page.
   */
  const canManageRoles = $derived(can(Permission.MANAGE_ROLES));

  /**
   * Builtin roles are compiled into the backend and cannot be deleted, so they
   * stay out of the selection entirely rather than failing on submit.
   */
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
  /** The role the form is editing; `null` means it is creating one. */
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
    /** A builtin role is never selectable — see `deletableRoleIds`. */
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

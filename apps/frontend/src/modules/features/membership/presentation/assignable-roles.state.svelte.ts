import { Permission, can, type PermissionScope } from '@platform/authz';
import { createQuery } from '@platform/query';
import { humanizeRoleId, roleQueries, type RoleEntity } from '@/modules/features/roles';

export interface AssignableRole {
  roleId: string;
  name: string;
  description: string;
  role?: RoleEntity;
}

/**
 * The roles a member view can hand out at `scope`. `ListGrantableRoles` has only the
 * builtins; the full catalog (custom roles too) needs `MANAGE_ROLES`, so it is asked
 * only when it would be answered.
 */
export const createAssignableRoles = (scope: PermissionScope) => {
  const canReadCatalog = $derived(can(Permission.MANAGE_ROLES));

  const catalogQuery = createQuery(() => roleQueries.catalog({ enabled: canReadCatalog }));
  const grantableQuery = createQuery(() => roleQueries.grantable(scope));

  const roles = $derived(catalogQuery.data ?? []);
  const grantableRoles = $derived(grantableQuery.data ?? []);

  /** Empty when the catalog is out of reach. */
  // Rebuilt whole by the `$derived` and never mutated after it is read, so a
  // reactive collection would only make a throwaway object track dependencies.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const roleById = $derived(new Map(roles.map(role => [role.id, role])));

  const assignableRoles = $derived.by((): AssignableRole[] => {
    // Rebuilt whole by the `$derived` and never mutated after it is read, so a
    // reactive collection would only make a throwaway object track dependencies.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const merged = new Map<string, AssignableRole>();

    for (const grantable of grantableRoles) {
      const role = roleById.get(grantable.roleId);
      merged.set(grantable.roleId, {
        roleId: grantable.roleId,
        name: role?.name ?? humanizeRoleId(grantable.roleId),
        description: role?.description || grantable.description,
        role,
      });
    }

    // Custom roles bound to this scope: grantable, but not in the static list.
    for (const role of roles) {
      if (role.scope !== scope || merged.has(role.id)) continue;
      merged.set(role.id, {
        roleId: role.id,
        name: role.name,
        description: role.description,
        role,
      });
    }

    return [...merged.values()];
  });

  // Rebuilt whole by the `$derived` and never mutated after it is read, so a
  // reactive collection would only make a throwaway object track dependencies.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const nameById = $derived(new Map(assignableRoles.map(entry => [entry.roleId, entry.name])));

  return {
    get assignableRoles() {
      return assignableRoles;
    },
    get roleById() {
      return roleById;
    },
    get isLoading() {
      return grantableQuery.isLoading;
    },
    /** Any role id, even one of another scope or no longer in the catalog. */
    labelFor: (roleId: string): string =>
      nameById.get(roleId) ?? roleById.get(roleId)?.name ?? humanizeRoleId(roleId),
  };
};

export type AssignableRoles = ReturnType<typeof createAssignableRoles>;

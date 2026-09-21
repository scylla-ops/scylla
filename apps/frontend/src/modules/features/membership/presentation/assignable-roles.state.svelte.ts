import { Permission, can, type PermissionScope } from '@platform/authz';
import { createQuery } from '@platform/query';
import { humanizeRoleId, roleQueries, type RoleEntity } from '@/modules/features/roles';

/** A role a member view may offer, whichever list it was found in. */
export interface AssignableRole {
  roleId: string;
  name: string;
  description: string;
  /** The catalog entry, when the caller may read the catalog. */
  role?: RoleEntity;
}

/**
 * The roles a member view may hand out at `scope`, and the labels for the ones
 * already held.
 *
 * Two lists back it, because no single one is both complete and readable by
 * everyone. `ListGrantableRoles` needs no permission but carries the builtins
 * only; the full catalog carries custom roles too but is gated behind
 * `MANAGE_ROLES`, which a tenant administrator does not hold. So the catalog is
 * requested only when it would be answered, and merged on top when it arrives —
 * a system administrator sees every role, everyone else sees the builtins, and
 * neither collects a denial for asking.
 */
export const createAssignableRoles = (scope: PermissionScope) => {
  // `can` is reactive here, so this recomputes the moment the permissions land
  // — which is what turns the catalog query on without an effect.
  const canReadCatalog = $derived(can(Permission.MANAGE_ROLES));

  const catalogQuery = createQuery(() => roleQueries.catalog({ enabled: canReadCatalog }));
  const grantableQuery = createQuery(() => roleQueries.grantable(scope));

  const roles = $derived(catalogQuery.data ?? []);
  const grantableRoles = $derived(grantableQuery.data ?? []);

  /** Catalog entries by id — empty when the catalog is out of reach. */
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

    // Custom roles bound to this scope: grantable, but never in the static list.
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
    /**
     * The display name of any role id, including one bound to another scope (an
     * organization role seen from a project view) or dropped from the catalog.
     */
    labelFor: (roleId: string): string =>
      nameById.get(roleId) ?? roleById.get(roleId)?.name ?? humanizeRoleId(roleId),
  };
};

export type AssignableRoles = ReturnType<typeof createAssignableRoles>;

import { useCallback, useMemo } from 'react';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  Permission,
  type PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';
import { useGrantableRoles } from '@/modules/features/permission/presentation/hooks/use-grantable-roles.ts';
import { useRoles } from '@/modules/features/permission/presentation/hooks/use-roles.ts';
import { humanizeRoleId } from '@/modules/features/permission/presentation/utils/role-label.ts';

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
export const useAssignableRoles = (scope: PermissionScope) => {
  const canReadCatalog = useCan(Permission.MANAGE_ROLES);
  const { roles } = useRoles({ enabled: canReadCatalog });
  const { grantableRoles, isLoading } = useGrantableRoles(scope);

  /** Catalog entries by id — empty when the catalog is out of reach. */
  const roleById = useMemo(() => new Map(roles.map(role => [role.id, role])), [roles]);

  const assignableRoles: AssignableRole[] = useMemo(() => {
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
      merged.set(role.id, { roleId: role.id, name: role.name, description: role.description, role });
    }

    return [...merged.values()];
  }, [grantableRoles, roleById, roles, scope]);

  const nameById = useMemo(
    () => new Map(assignableRoles.map(entry => [entry.roleId, entry.name])),
    [assignableRoles],
  );

  /**
   * The display name of any role id, including one bound to another scope (an
   * organization role seen from a project view) or dropped from the catalog.
   */
  const labelFor = useCallback(
    (roleId: string): string =>
      nameById.get(roleId) ?? roleById.get(roleId)?.name ?? humanizeRoleId(roleId),
    [nameById, roleById],
  );

  return { assignableRoles, labelFor, roleById, isLoading };
};

import { useCallback, useMemo } from 'react';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useGrants } from '@/modules/features/permission/presentation/hooks/use-grants.ts';
import { useRoles } from '@/modules/features/permission/presentation/hooks/use-roles.ts';

/**
 * Why a user may or may not receive a project-scoped grant in an organization.
 *
 * - `eligible`            — the grant will work and the user will reach the project.
 * - `not-admitted`        — no grant bound to the organization. The backend's
 *                           tenant boundary rejects the grant outright.
 * - `cannot-see-projects` — admitted, but nothing they hold lets them read the
 *                           organization, so the project list stays closed to
 *                           them and the grant would be dead weight.
 */
export type GrantEligibility = 'eligible' | 'not-admitted' | 'cannot-see-projects';

/**
 * Whether a role confers `permission`.
 *
 * An access arm this build cannot read — or a role missing from the catalog —
 * counts as conferring. That is the opposite of `canAccess`, deliberately: there,
 * denying on the unknown protects the UI; here, it would lock an administrator
 * out of granting for a reason they cannot see or fix. The backend stays the
 * enforcer either way.
 */
const roleConfers = (role: RoleEntity | undefined, permission: Permission): boolean => {
  if (!role) return true;
  switch (role.access.kind) {
    case 'fullControl':
      return true;
    case 'restricted':
      return role.access.permissions.includes(permission);
    default:
      return true;
  }
};

/**
 * Answers, for one organization, whether each user can usefully receive a
 * project-scoped grant in it — so the picker can grey out the others and say why
 * instead of failing server-side.
 */
export const useProjectGrantEligibility = (organizationId: string | null) => {
  const { grants } = useGrants();
  const { roles } = useRoles();

  const roleById = useMemo(() => new Map(roles.map(role => [role.id, role])), [roles]);

  /**
   * Grants bound to this organization's own scope, by user. A system-wide grant
   * is not one: the backend looks for `Scope::Organization(org)` exactly.
   */
  const orgGrantsByUser = useMemo(() => {
    const byUser = new Map<string, string[]>();
    if (!organizationId) return byUser;
    for (const grant of grants) {
      if (
        grant.principal.kind !== PrincipalKind.USER ||
        grant.scope !== PermissionScope.ORGANIZATION ||
        grant.scopeId !== organizationId
      ) {
        continue;
      }
      byUser.set(grant.principal.id, [...(byUser.get(grant.principal.id) ?? []), grant.roleId]);
    }
    return byUser;
  }, [grants, organizationId]);

  const eligibilityFor = useCallback(
    (userId: string): GrantEligibility => {
      const roleIds = orgGrantsByUser.get(userId);
      if (!roleIds || roleIds.length === 0) return 'not-admitted';

      const canReadOrganization = roleIds.some(roleId =>
        roleConfers(roleById.get(roleId), Permission.READ_ORGANIZATION),
      );
      return canReadOrganization ? 'eligible' : 'cannot-see-projects';
    },
    [orgGrantsByUser, roleById],
  );

  return { eligibilityFor };
};

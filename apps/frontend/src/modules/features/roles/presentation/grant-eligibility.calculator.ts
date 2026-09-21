import { Permission, PermissionScope, PrincipalKind } from '@platform/authz';
import { roleConfers, type RoleEntity } from '../domain/entities/role.entity.ts';
import type { GrantEntity } from '../domain/entities/grant.entity.ts';

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
 * Answers, for one organization, whether each user can usefully receive a
 * project-scoped grant in it — so the picker can grey the others out and say
 * why, instead of failing server-side.
 *
 * Pure, and deliberately: it is a fold over two lists the dialog already holds,
 * it is the only part of the grant picker with a rule in it, and a test can pin
 * it without a DOM.
 */
export const buildGrantEligibility = (
  grants: readonly GrantEntity[],
  roles: readonly RoleEntity[],
  organizationId: string | null,
): ((userId: string) => GrantEligibility) => {
  const roleById = new Map(roles.map(role => [role.id, role]));

  /**
   * Grants bound to this organization's own scope, by user. A system-wide grant
   * is not one: the backend looks for `Scope::Organization(org)` exactly.
   */
  const roleIdsByUser = new Map<string, string[]>();
  if (organizationId) {
    for (const grant of grants) {
      if (
        grant.principal.kind !== PrincipalKind.USER ||
        grant.scope !== PermissionScope.ORGANIZATION ||
        grant.scopeId !== organizationId
      ) {
        continue;
      }
      roleIdsByUser.set(grant.principal.id, [
        ...(roleIdsByUser.get(grant.principal.id) ?? []),
        grant.roleId,
      ]);
    }
  }

  return (userId: string): GrantEligibility => {
    const roleIds = roleIdsByUser.get(userId);
    if (!roleIds || roleIds.length === 0) return 'not-admitted';

    const canReadOrganization = roleIds.some(roleId =>
      roleConfers(roleById.get(roleId), Permission.READ_ORGANIZATION),
    );
    return canReadOrganization ? 'eligible' : 'cannot-see-projects';
  };
};

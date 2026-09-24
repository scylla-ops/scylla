import type { GrantEntity } from '@/modules/features/roles';
import { PermissionScope, PrincipalKind, } from '@platform/authz';

/** `INHERITED`: from an enclosing scope (an organization role reaching a project). It cannot be edited from the narrower view. */
export enum MemberRoleOrigin {
  DIRECT = 'direct',
  INHERITED = 'inherited',
}

export interface MemberRole {
  grantId: string;
  roleId: string;
  origin: MemberRoleOrigin;
  scope: PermissionScope;
}

/** Built from grants: on the backend, holding a grant is belonging. */
export interface ScopeMember {
  userId: string;
  roles: MemberRole[];
}

const userGrants = (grants: GrantEntity[]): GrantEntity[] =>
  grants.filter(grant => grant.principal.kind === PrincipalKind.USER);

/** Users known to belong but without an organization role are still listed, with no roles. */
export const buildOrganizationMembers = (
  organizationGrants: GrantEntity[],
  knownUserIds: string[] = [],
): ScopeMember[] => {
  const byUser = new Map<string, ScopeMember>();
  const ensure = (userId: string): ScopeMember => {
    const existing = byUser.get(userId);
    if (existing) return existing;
    const member: ScopeMember = { userId, roles: [] };
    byUser.set(userId, member);
    return member;
  };

  for (const userId of knownUserIds) ensure(userId);

  for (const grant of userGrants(organizationGrants)) {
    ensure(grant.principal.id).roles.push({
      grantId: grant.id,
      roleId: grant.roleId,
      origin: MemberRoleOrigin.DIRECT,
      scope: PermissionScope.ORGANIZATION,
    });
  }

  return [...byUser.values()];
};

/**
 * Each member once, with the project's own roles and the organization roles that
 * reach it. `organizationRoleReachesProjects` leaves out the roles that confer
 * nothing on a project. `knownUserIds` lists people whose grants the caller may not read.
 */
export const buildProjectMembers = (
  projectGrants: GrantEntity[],
  organizationGrants: GrantEntity[],
  organizationRoleReachesProjects: (roleId: string) => boolean,
  knownUserIds: string[] = [],
): ScopeMember[] => {
  const byUser = new Map<string, ScopeMember>();
  const ensure = (userId: string): ScopeMember => {
    const existing = byUser.get(userId);
    if (existing) return existing;
    const member: ScopeMember = { userId, roles: [] };
    byUser.set(userId, member);
    return member;
  };

  for (const userId of knownUserIds) ensure(userId);

  for (const grant of userGrants(projectGrants)) {
    ensure(grant.principal.id).roles.push({
      grantId: grant.id,
      roleId: grant.roleId,
      origin: MemberRoleOrigin.DIRECT,
      scope: PermissionScope.PROJECT,
    });
  }

  for (const grant of userGrants(organizationGrants)) {
    if (!organizationRoleReachesProjects(grant.roleId)) continue;
    ensure(grant.principal.id).roles.push({
      grantId: grant.id,
      roleId: grant.roleId,
      origin: MemberRoleOrigin.INHERITED,
      scope: PermissionScope.ORGANIZATION,
    });
  }

  return [...byUser.values()];
};

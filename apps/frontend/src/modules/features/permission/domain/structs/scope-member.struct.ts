import type { GrantEntity } from '@/modules/features/permission/domain/entities/grant.entity.ts';
import {
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * One role a member holds over a scope, and where it comes from.
 *
 * `DIRECT` is a grant bound to the scope being looked at. `INHERITED` is a
 * grant bound to an enclosing scope that reaches into it — an organization role
 * covering every project of the organization. The distinction is not cosmetic:
 * an inherited role cannot be edited from the narrower view, because the grant
 * it comes from does not live there.
 */
export enum MemberRoleOrigin {
  DIRECT = 'direct',
  INHERITED = 'inherited',
}

export interface MemberRole {
  /** The grant conferring it — what a revoke acts on. */
  grantId: string;
  roleId: string;
  origin: MemberRoleOrigin;
  /** The scope the grant is bound to, which is what the scope badge shows. */
  scope: PermissionScope;
}

/**
 * A user seen once, with every role they hold over the scope — a value object
 * assembled from grants, not a stored record. Membership has no storage of its
 * own on the backend: holding a grant *is* belonging.
 */
export interface ScopeMember {
  userId: string;
  roles: MemberRole[];
}

/** Sole holder of the "which grants are users" rule, applied before anything else. */
const userGrants = (grants: GrantEntity[]): GrantEntity[] =>
  grants.filter(grant => grant.principal.kind === PrincipalKind.USER);

/**
 * The members of an organization: everyone holding a grant bound to it.
 *
 * Users known to belong but holding no organization-scoped role (they were
 * reached through a project, or the caller cannot read the grants) are still
 * listed, with an empty role list — the member list answers "who is here", and
 * an empty row says so more usefully than an omission.
 */
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
 * The members of a project, each listed once, carrying the roles they hold on
 * it directly *and* the organization roles that reach into it.
 *
 * `organizationRoleReachesProjects` decides which organization grants count.
 * Every organization role technically covers the projects beneath it — the
 * backend's scope hierarchy sees to that — but a role conferring nothing there
 * (the `organization-member` floor: "belongs here, sees it exists") would
 * appear as project access that does not exist. The caller supplies the test
 * because answering it needs the role catalog, which not every caller may read;
 * an unreadable role is treated as reaching, so access is never hidden.
 *
 * `knownUserIds` seeds the list the same way the organization builder does, and
 * for the same reason: reading the grants needs `MANAGE_PROJECT_GRANTS`, which
 * someone merely allowed to *see* the members does not hold. Without the seed
 * the project would look deserted to them; with it they get the people, and an
 * empty role list saying the roles are none of their business.
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

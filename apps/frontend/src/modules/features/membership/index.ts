/**
 * Who belongs to an organization or a project, and with which roles.
 *
 * Membership is its own feature rather than a corner of `organization` and
 * `project` because both scopes show the same thing about a different subject —
 * it used to be duplicated deep-import surface shared by two pages. It composes
 * `permission` (role and grant data), `organization`, `project` and `user`, and
 * nothing depends on it except the router.
 */
export {
  type ScopeMember,
  type MemberRole,
  MemberRoleOrigin,
  buildOrganizationMembers,
  buildProjectMembers,
} from './domain/structs/scope-member.struct.ts';
export { useScopeMembership } from './presentation/hooks/use-scope-membership.ts';
export { useAssignableRoles } from './presentation/hooks/use-assignable-roles.ts';
export { OrganizationMembersPage } from './presentation/ui/OrganizationMembers.page.tsx';
export { ProjectMembersPage } from './presentation/ui/ProjectMembers.page.tsx';

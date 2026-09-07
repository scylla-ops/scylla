/**
 * Who belongs to an organization or a project, and with which roles.
 *
 * Membership is its own feature rather than a corner of `organization` and
 * `project` because both scopes show the same thing about a different subject.
 * It composes `roles`, `organization`, `project` and `user` through their
 * public APIs, and nothing depends on it except the router.
 *
 * Its two pages are deliberately *not* exported: `membership.module.ts` loads
 * them lazily, and a barrel that re-exported them would pull them into the
 * chunk of anything importing this module.
 */
export {
  type ScopeMember,
  type MemberRole,
  MemberRoleOrigin,
  buildOrganizationMembers,
  buildProjectMembers,
} from './domain/structs/scope-member.struct.ts';
export { useScopeMembership } from './presentation/hooks/use-scope-membership.ts';
export {
  useAssignableRoles,
  type AssignableRole,
} from './presentation/hooks/use-assignable-roles.ts';

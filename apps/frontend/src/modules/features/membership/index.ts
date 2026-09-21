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
 *
 * The two hooks that used to be here are gone with the Phase 3 port: they were
 * view models, not API, and nothing outside this feature ever called them. What
 * remains is the pure domain — which is what another module would actually want.
 */
export {
  type ScopeMember,
  type MemberRole,
  MemberRoleOrigin,
  buildOrganizationMembers,
  buildProjectMembers,
} from './domain/structs/scope-member.struct.ts';

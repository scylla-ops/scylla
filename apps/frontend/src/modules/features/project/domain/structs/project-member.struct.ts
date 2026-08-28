/**
 * Someone holding a role scoped to the project, as the backend lists them.
 *
 * Deliberately narrower than what the members view shows: `ListProjectMembers`
 * answers with the holders of a *project-scoped* grant only — people reaching
 * the project through an organization role are not in it. Assembling the
 * unified list is the caller's job (see `buildProjectMembers`); this struct is
 * the name resolution that goes with the project-scoped half of it.
 *
 * A value object, not an entity: it carries no identity of its own beyond the
 * user it points at.
 */
export interface ProjectMember {
  userId: string;
  username: string;
}

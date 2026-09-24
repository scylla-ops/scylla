/** The holders of a project-scoped grant only; `buildProjectMembers` adds the organization roles. */
export interface ProjectMember {
  userId: string;
  username: string;
}

/**
 * Projects: the unit that owns pipelines, secrets and its own member list.
 *
 * The public API of the module. The three ways an organization's projects get
 * read — the paginated list, the dashboard overview and the grant-label lookup
 * — all go through here and therefore share one cache entry.
 */
export type { ProjectEntity } from './domain/entities/project.entity.ts';
export type { ProjectMember } from './domain/structs/project-member.struct.ts';
export { PROJECTS_QUERY_KEY } from './presentation/hooks/projects.query-keys.ts';
export { useProjects } from './presentation/hooks/useProjects.ts';
export { useOrganizationProjects } from './presentation/hooks/use-organization-projects.ts';
export {
  useProjectsByOrganizations,
  type ProjectLookupEntry,
} from './presentation/hooks/use-projects-by-organizations.ts';
export {
  useProjectMembers,
  PROJECT_MEMBERS_QUERY_KEY,
} from './presentation/hooks/use-project-members.ts';

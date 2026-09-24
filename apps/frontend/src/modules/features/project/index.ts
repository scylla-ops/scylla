/** The list, the overview and the lookup share one cache entry through `projectQueries`. */
export type { ProjectEntity } from './domain/entities/project.entity.ts';
export type { ProjectMember } from './domain/structs/project-member.struct.ts';
export {
  canListProjects,
  invalidateProjectMembers,
  projectLookupQueries,
  projectMutations,
  projectQueries,
  type ProjectLookupEntry,
  PROJECTS_LOOKUP_PAGE,
  PROJECTS_QUERY_KEY,
  PROJECTS_QUERY_ROOT,
  PROJECT_MEMBERS_QUERY_KEY,
} from './presentation/project.queries.ts';

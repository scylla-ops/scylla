/**
 * Projects: the unit that owns pipelines, secrets and its own member list.
 *
 * The three ways an organization's projects get read — the paginated list, the
 * dashboard overview and the grant-label lookup — all go through
 * `projectQueries` and therefore share one cache entry. They are plain options
 * objects, so the modules still on React run them through react-query while
 * this one runs them through `createQuery`.
 */
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

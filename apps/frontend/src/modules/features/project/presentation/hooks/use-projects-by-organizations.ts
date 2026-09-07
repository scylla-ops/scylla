import { useQueries } from '@tanstack/react-query';
import { useProjectDomain } from '@/modules/features/project/presentation/hooks/use-project-domain.ts';
import {
  PROJECTS_LOOKUP_PAGE,
  PROJECTS_QUERY_KEY,
} from '@/modules/features/project/presentation/hooks/projects.query-keys.ts';

/** Where a project lives, for callers that only hold a project id. */
export interface ProjectLookupEntry {
  name: string;
  organizationId: string;
}

/**
 * A projectId -> {name, organization} lookup across several organizations.
 *
 * Part of the module's public API: a project id on its own says nothing about
 * which organization owns it, so anything resolving ids to names (grant target
 * labels, for one) has to fan out over the user's organizations. That fan-out
 * is a project concern, so it lives here instead of being rebuilt by each
 * caller against `projectRepository`.
 *
 * Shares {@link PROJECTS_QUERY_KEY} with the paginated list, so an organization
 * already loaded elsewhere is served from cache rather than refetched.
 */
export const useProjectsByOrganizations = (organizationIds: string[], enabled = true) => {
  const { projectRepository } = useProjectDomain();

  return useQueries({
    queries: enabled
      ? organizationIds.map(organizationId => ({
          queryKey: PROJECTS_QUERY_KEY(organizationId, PROJECTS_LOOKUP_PAGE),
          queryFn: async () =>
            (
              await projectRepository.getByOrganizationId(organizationId, PROJECTS_LOOKUP_PAGE)
            ).unwrap(),
          staleTime: 30_000,
        }))
      : [],
    // Folded here rather than by the caller so TanStack Query can memoize the
    // map on the underlying results instead of rebuilding it every render.
    combine: results => {
      const byProjectId = new Map<string, ProjectLookupEntry>();
      results.forEach((result, index) => {
        const organizationId = organizationIds[index];
        for (const project of result.data?.projects ?? []) {
          byProjectId.set(project.id, { name: project.name, organizationId });
        }
      });
      return byProjectId;
    },
  });
};

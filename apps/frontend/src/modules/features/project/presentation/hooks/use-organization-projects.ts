import { useQuery } from '@tanstack/react-query';
import { useProjectDomain } from '@/modules/features/project/presentation/hooks/use-project-domain.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import {
  PROJECTS_LOOKUP_PAGE,
  PROJECTS_QUERY_KEY,
} from '@/modules/features/project/presentation/hooks/projects.query-keys.ts';

/**
 * Every project of one organization, unpaginated from the caller's point of
 * view — for the pages that need the whole list rather than a page of it.
 *
 * Part of the module's public API: other features need an organization's
 * projects (the dashboard overview does) and must get them from here rather
 * than reaching for `projectRepository` themselves, which would fork the cache.
 *
 * The backend already filters to what the caller may read, so there is no
 * client-side permission gate here.
 */
export const useOrganizationProjects = (organizationId: string | null) => {
  const { projectRepository } = useProjectDomain();

  const { data, isLoading, isError } = useQuery({
    queryKey: PROJECTS_QUERY_KEY(organizationId, PROJECTS_LOOKUP_PAGE),
    queryFn: async () =>
      (await projectRepository.getByOrganizationId(organizationId!, PROJECTS_LOOKUP_PAGE)).unwrap(),
    enabled: !!organizationId,
    staleTime: 30_000,
  });

  const projects: ProjectEntity[] = data?.projects ?? [];

  return { projects, isLoading, isError };
};

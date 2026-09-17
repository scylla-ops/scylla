import { useEffect } from 'react';
import { useQuery, keepPreviousData } from '@tanstack/react-query';
import { useProjectDomain } from '@/modules/features/project/presentation/hooks/use-project-domain.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';
import { PROJECTS_QUERY_KEY } from '@/modules/features/project/presentation/hooks/projects.query-keys.ts';
import { Permission, useAuthorization } from '@platform/authz';

/**
 * One organization's projects, paginated.
 *
 * The backend checks `ListProjectsByOrganization` on the organization, so the
 * query asks for itself: the hook is exported through the barrel and `roles`
 * calls it from a page entered on `MANAGE_ROLES`, well outside this module's
 * route guard. `canListProjects` is returned because an empty list otherwise
 * reads as "no projects" when it means "not allowed to look".
 */
export const useProjects = (organizationId: string | null) => {
  const { projectRepository } = useProjectDomain();
  const { page, setPage, paginationParams } = usePagination();
  const { can, ready } = useAuthorization();

  const canListProjects =
    ready && !!organizationId && can(Permission.LIST_PROJECTS_BY_ORGANIZATION, { organizationId });

  const { data, isLoading, error } = useQuery({
    queryKey: PROJECTS_QUERY_KEY(organizationId, paginationParams),
    queryFn: async () => (await projectRepository.getByOrganizationId(organizationId!, paginationParams)).unwrap(),
    enabled: canListProjects,
    placeholderData: keepPreviousData,
  });

  useEffect(() => {
    setPage(1);
  }, [organizationId, setPage]);

  return {
    projects: data?.projects,
    paginationInfo: data?.pagination,
    page,
    setPage,
    isLoading,
    isError: !!error,
    /** False also means "denied" — an empty `projects` is not proof of none. */
    canListProjects,
  };
};

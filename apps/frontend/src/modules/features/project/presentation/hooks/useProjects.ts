import { useEffect } from 'react';
import { useQuery, keepPreviousData } from '@tanstack/react-query';
import { useProjectDomain } from '@/modules/features/project/presentation/hooks/use-project-domain.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';

export const useProjects = (organizationId: string | null) => {
  const { projectRepository } = useProjectDomain();
  const { page, setPage, paginationParams } = usePagination();

  const { data, isLoading, error } = useQuery({
    queryKey: ['projects', organizationId, paginationParams],
    queryFn: async () => (await projectRepository.getByOrganizationId(organizationId!, paginationParams)).unwrap(),
    enabled: !!organizationId,
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
  };
};

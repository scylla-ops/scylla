import { useEffect } from 'react';
import { useQuery, keepPreviousData } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';
import type { PaginationInfo } from '@/modules/shared/domain/types/Pagination.ts';

export const useProjects = (organizationId: string | null) => {
  const { getProjects } = useDependencies().project;
  const { page, setPage, paginationParams } = usePagination();

  const { data, isLoading, error } = useQuery({
    queryKey: ['projects', organizationId, paginationParams],
    queryFn: async () => (await getProjects.execute(organizationId!, paginationParams)).unwrap(),
    enabled: !!organizationId,
    placeholderData: keepPreviousData,
  });

  useEffect(() => {
    setPage(1);
  }, [organizationId, setPage]);

  return {
    projects: data?.projects,
    paginationInfo: data?.pagination as PaginationInfo | undefined,
    page,
    setPage,
    isLoading,
    isError: !!error,
  };
};

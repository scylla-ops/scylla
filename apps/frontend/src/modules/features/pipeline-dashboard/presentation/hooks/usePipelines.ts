import { useQuery, keepPreviousData } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { usePagination } from '@/modules/shared/presentation/hooks/usePagination.ts';
import type { PaginationInfo } from '@/modules/shared/domain/types/Pagination.ts';

export const usePipelines = (projectId: string) => {
  const { getPipelines } = useDependencies().pipelineDashboard;
  const { page, setPage, paginationParams } = usePagination();

  const {
    data,
    isLoading,
    error,
    isError,
  } = useQuery<ListPipelinesResponse, ScyllaError>({
    queryKey: ['pipelines', projectId, paginationParams],
    queryFn: async () => (await getPipelines.execute(projectId, paginationParams)).unwrap(),
    staleTime: 0,
    gcTime: 0,
    placeholderData: keepPreviousData,
  });

  return {
    pipelines: data?.pipelines,
    paginationInfo: data?.pagination as PaginationInfo | undefined,
    page,
    setPage,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
  };
};

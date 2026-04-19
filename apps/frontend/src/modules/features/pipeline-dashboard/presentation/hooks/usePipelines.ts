import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { usePagination } from '@/modules/shared/presentation/hooks/usePagination.ts';
import type { PaginationInfo } from '@/modules/shared/domain/types/Pagination.ts';
import { useEffect } from 'react';

export const usePipelines = (projectId: string) => {
  const { getPipelines } = useDependencies().pipelineDashboard;
  const { paginationParams, paginationInfo, updatePaginationInfo, setPage } = usePagination();

  const { data, isLoading, error, isError } = useQuery<ListPipelinesResponse, ScyllaError>({
    queryKey: ['pipelines', projectId, paginationParams],
    queryFn: async () => (await getPipelines.execute(projectId, paginationParams)).unwrap(),
    staleTime: 0,
  });

  useEffect(() => {
    updatePaginationInfo(data?.pagination as PaginationInfo | undefined);
  }, [data, updatePaginationInfo]);

  return {
    pipelines: data?.pipelines,
    paginationInfo,
    setPage,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
  };
};

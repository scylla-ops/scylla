import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';
import { useEffect } from 'react';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';

export const usePipelines = (projectId: string) => {
  const { getPipelines } = useDependencies().pipeline;
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

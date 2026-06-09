import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';
import { useEffect } from 'react';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export const usePipelinesMetadata = (projectId: string) => {
  const { getPipelinesMetadata } = useDependencies().pipeline;
  const { paginationParams, paginationInfo, updatePaginationInfo, setPage } = usePagination();

  const { data, isLoading, error, isError } = useQuery<
    PaginatedList<PipelineMetadata>,
    ScyllaError
  >({
    queryKey: ['pipelines', projectId, paginationParams],
    queryFn: async () => (await getPipelinesMetadata.execute(projectId, paginationParams)).unwrap(),
    staleTime: 5 * 1000, // 5 seconds //todo: more long and refresh button?
  });

  useEffect(() => {
    updatePaginationInfo(data?.pagination);
  }, [data, updatePaginationInfo]);

  return {
    pipelines: data,
    paginationInfo,
    setPage,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
  };
};

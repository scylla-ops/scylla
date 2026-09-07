import { useQuery } from '@tanstack/react-query';
import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';
import { useEffect } from 'react';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

export const usePipelinesMetadata = (projectId: string) => {
  const { pipelineRepository } = usePipelineDomain();
  const { paginationParams, paginationInfo, updatePaginationInfo, setPage } = usePagination();

  const { data, isLoading, error, isError } = useQuery<
    PaginatedList<PipelineMetadata>,
    ScyllaError
  >({
    queryKey: ['pipelines', projectId, paginationParams],
    queryFn: async () => (await pipelineRepository.getMetadataByProjectId(projectId, paginationParams)).unwrap(),
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

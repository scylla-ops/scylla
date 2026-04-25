import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListJobsResponse } from '@/generated/job.ts';
import { usePagination } from '@/modules/shared/presentation/hooks/usePagination.ts';
import { useEffect } from 'react';
import type { PaginationInfo } from '@shared/domain/models/Pagination.ts';

export const usePipelineJobs = (pipelineId: string) => {
  const { getPipelineJobs } = useDependencies().jobs;
  const { paginationParams, paginationInfo, updatePaginationInfo, setPage } = usePagination();

  const { data, isLoading, error, isError, refetch } = useQuery<ListJobsResponse, ScyllaError>({
    queryKey: ['jobs', 'pipeline', pipelineId, paginationParams],
    queryFn: async () => (await getPipelineJobs.execute(pipelineId, paginationParams)).unwrap(),
    enabled: !!pipelineId,
    staleTime: 0, //TODO: check if need cache
    refetchInterval: query => {
      const jobs = query.state.data?.jobs || [];
      const hasActiveJobs = jobs.some(job => job.status === 'running' || job.status === 'pending');
      return hasActiveJobs ? 5000 : false;
    },
  });

  useEffect(() => {
    updatePaginationInfo(data?.pagination as PaginationInfo | undefined);
  }, [data, updatePaginationInfo]);

  return {
    jobs: data?.jobs,
    paginationInfo,
    setPage,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
    refetch,
  };
};

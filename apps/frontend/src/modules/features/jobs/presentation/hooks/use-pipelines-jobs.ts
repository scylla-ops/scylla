import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { usePagination } from '@shared/presentation/hooks/use-pagination.ts';
import { useEffect } from 'react';
import { JOBS_QUERY_KEY } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-jobs.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

export const usePipelinesJobs = (pipelineId: string) => {
  const { getPipelineJobs } = useDependencies().jobs;
  const { paginationParams, paginationInfo, updatePaginationInfo, setPage } = usePagination();

  const { data, isLoading, error, isError, refetch } = useQuery<
    PaginatedList<JobEntity>,
    ScyllaError
  >({
    queryKey: [...JOBS_QUERY_KEY(pipelineId), paginationParams],
    queryFn: async () => (await getPipelineJobs.execute(pipelineId, paginationParams)).unwrap(),
    enabled: !!pipelineId,
    staleTime: 0,
    refetchInterval: query => {
      const jobs = query.state.data?.items || [];
      const hasActiveJobs = jobs.some(job => job.status === 'running' || job.status === 'pending');
      return hasActiveJobs ? 5000 : false;
    },
  });

  useEffect(() => {
    updatePaginationInfo(data?.pagination);
  }, [data, updatePaginationInfo]);

  return {
    jobs: data?.items,
    paginationInfo,
    setPage,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
    refetch,
  };
};

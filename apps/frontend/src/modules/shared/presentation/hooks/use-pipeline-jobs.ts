import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';

type UsePipelineJobsOptions = {
  limit?: number;
  enabled?: boolean;
};

/**
 * Hook to fetch jobs for a specific pipeline
 * Shared hook that can be used by both jobs and pipeline-dashboard modules
 * @param pipelineId - Pipeline ID to fetch jobs for
 * @param options - Optional configuration (limit, enabled)
 * @returns Jobs data, loading state, error state, and refetch function
 */
export const usePipelineJobs = (pipelineId: string, options?: UsePipelineJobsOptions) => {
  const { getPipelineJobs } = useDependencies().jobs;
  const { limit, enabled = true } = options || {};

  const { data, isLoading, error, isError, refetch } = useQuery<PaginatedList<Job>, ScyllaError>({
    queryKey: ['jobs', 'pipeline', pipelineId, limit],
    queryFn: async () => {
      const pagination = limit ? { page: 1, pageSize: limit } : undefined;
      return (await getPipelineJobs.execute(pipelineId, pagination)).unwrap();
    },
    enabled: !!pipelineId && enabled,
    staleTime: 0, // Data is considered fresh for 5 seconds
    refetchInterval: query => {
      // Auto-refresh every 5 seconds if there are running/pending jobs
      const jobs = query.state.data?.items || [];
      const hasActiveJobs = jobs.some(job => job.status === 'running' || job.status === 'pending');
      return hasActiveJobs ? 5000 : false;
    },
  });

  return {
    jobs: data?.items || [],
    pagination: data?.pagination,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
    refetch,
  };
};

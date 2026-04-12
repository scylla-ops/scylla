import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListJobsResponse } from '@/generated/job.ts';

export const usePipelineJobs = (pipelineId: string) => {
  const { getPipelineJobs } = useDependencies().jobs;

  const {
    data,
    isLoading,
    error,
    isError,
    refetch,
  } = useQuery<ListJobsResponse, ScyllaError>({
    queryKey: ['jobs', 'pipeline', pipelineId],
    queryFn: async () => (await getPipelineJobs.execute(pipelineId)).unwrap(),
    enabled: !!pipelineId,
    staleTime: 0,
    gcTime: 0,
    refetchInterval: (query) => {
      // Auto-refresh every 5 seconds if there are running/pending jobs
      const jobs = query.state.data?.jobs || [];
      const hasActiveJobs = jobs.some(
        job => job.status === 'running' || job.status === 'pending',
      );
      return hasActiveJobs ? 5000 : false;
    },
  });

  return {
    jobs: data?.jobs,
    pagination: data?.pagination,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
    refetch,
  };
};


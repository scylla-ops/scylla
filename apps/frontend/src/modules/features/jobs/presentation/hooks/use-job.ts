import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';

export const useJob = (jobId: string) => {
  const { getJobById } = useDependencies().jobs;

  const {
    data: job,
    isLoading,
    error,
    isError,
  } = useQuery<Job, ScyllaError>({
    queryKey: ['jobs', jobId],
    queryFn: async () => (await getJobById.execute(jobId)).unwrap(),
    enabled: !!jobId,
    refetchInterval: query => {
      const status = query.state.data?.status;
      if (!status || status === 'completed' || status === 'failed') return false;
      return 3000;
    },
  });

  return {
    job,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
  };
};

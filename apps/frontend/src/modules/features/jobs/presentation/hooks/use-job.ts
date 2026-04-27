import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { JobResponse } from '@/generated/job.ts';

export const useJob = (jobId: string) => {
  const { getJobById } = useDependencies().jobs;

  const {
    data: job,
    isLoading,
    error,
    isError,
  } = useQuery<JobResponse, ScyllaError>({
    queryKey: ['jobs', jobId],
    queryFn: async () => (await getJobById.execute(jobId)).unwrap(),
    enabled: !!jobId,
  });

  return {
    job,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
  };
};

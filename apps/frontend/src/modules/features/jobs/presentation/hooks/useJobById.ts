import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import type { JobResponse } from '@/generated/job.ts';

export const useJobById = (jobId: string) => {
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


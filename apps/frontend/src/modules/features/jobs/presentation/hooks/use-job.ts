import { useQuery } from '@tanstack/react-query';
import { useJobsDomain } from '@/modules/features/jobs/presentation/hooks/use-jobs-domain.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

export const useJob = (jobId: string) => {
  const { jobsRepository } = useJobsDomain();

  const {
    data: job,
    isLoading,
    error,
    isError,
  } = useQuery<JobEntity, ScyllaError>({
    queryKey: ['jobs', jobId],
    queryFn: async () => (await jobsRepository.getById(jobId)).unwrap(),
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

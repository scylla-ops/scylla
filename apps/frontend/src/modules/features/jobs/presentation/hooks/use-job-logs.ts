import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { JobLog } from '@/modules/features/jobs/domain/models/job.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

export const useJobLogs = (jobId: string) => {
  const { listJobLogs } = useDependencies().jobs;

  const {
    data: logs,
    isLoading,
    isError,
  } = useQuery<PaginatedList<JobLog>, ScyllaError>({
    queryKey: ['job-logs'],
    queryFn: async () => (await listJobLogs.execute(jobId)).unwrap(),
    staleTime: 1000 * 3,
  });

  return {
    logs,
    isLoading,
    isError,
  };
};

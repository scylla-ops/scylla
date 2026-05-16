import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { JobLog } from '@/modules/features/jobs/domain/models/job.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

export const useJobLogs = (jobId: string, nodeId?: string) => {
  const { listJobLogs } = useDependencies().jobs;

  const {
    data: logs,
    isLoading,
    isError,
  } = useQuery<PaginatedList<JobLog>, ScyllaError>({
    queryKey: ['job-logs', jobId, nodeId],
    queryFn: async () => (await listJobLogs.execute(jobId, nodeId)).unwrap(),
    staleTime: 1000 * 2,
  });

  return {
    logs,
    isLoading,
    isError,
  };
};

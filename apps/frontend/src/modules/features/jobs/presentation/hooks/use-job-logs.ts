import { useQuery } from '@tanstack/react-query';
import { useJobsDomain } from '@/modules/features/jobs/presentation/hooks/use-jobs-domain.ts';
import type { JobLog } from '@/modules/features/jobs/domain/structs/job.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

export const useJobLogs = (jobId: string, nodeId?: string) => {
  const { jobsRepository } = useJobsDomain();

  const {
    data: logs,
    isLoading,
    isError,
  } = useQuery<PaginatedList<JobLog>, ScyllaError>({
    queryKey: ['job-logs', jobId, nodeId],
    queryFn: async () => (await jobsRepository.getLogs(jobId, nodeId)).unwrap(),
    staleTime: 1000 * 2,
  });

  return {
    logs,
    isLoading,
    isError,
  };
};

import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobLog } from '@/modules/features/jobs/domain/models/job.model.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';

export class ListJobLogsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobLog>>> {
    return this.repository.getLogs(jobId, nodeId, pagination);
  }
}

import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListJobLogsResponse } from '@/generated/job.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class ListJobLogsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobLogsResponse>> {
    return this.repository.getLogs(jobId, nodeId, pagination);
  }
}

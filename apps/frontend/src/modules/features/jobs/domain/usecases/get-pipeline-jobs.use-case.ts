import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListJobsResponse } from '@/generated/job.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class GetPipelineJobsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobsResponse>> {
    return this.repository.getByPipelineId(pipelineId, pagination);
  }
}

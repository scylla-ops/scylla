import type { JobsRepository } from '@/modules/features/jobs/domain/repository/JobsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListJobsResponse } from '@/generated/job.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class GetPipelineJobs {
  constructor(private readonly repository: JobsRepository) {}

  public execute(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobsResponse>> {
    return this.repository.getByPipelineId(pipelineId, pagination);
  }
}


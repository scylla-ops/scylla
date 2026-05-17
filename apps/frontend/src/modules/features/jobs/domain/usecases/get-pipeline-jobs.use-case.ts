import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';

export class GetPipelineJobsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<Job>>> {
    return this.repository.getByPipelineId(pipelineId, pagination);
  }
}

import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export class GetPipelineJobsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobEntity>>> {
    return this.repository.getByPipelineId(pipelineId, pagination);
  }
}

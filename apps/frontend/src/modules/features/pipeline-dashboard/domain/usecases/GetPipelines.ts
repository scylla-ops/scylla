import type { PipelineDashboardRepository } from '@/modules/features/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class GetPipelines {
  constructor(private readonly pipelineDashboardRepository: PipelineDashboardRepository) {}

  public async execute(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.pipelineDashboardRepository.getByProjectId(projectId, pagination);
  }
}

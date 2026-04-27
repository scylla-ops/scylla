import type { PipelineRepository } from '@/modules/features/pipeline-dashboard/domain/repository/pipeline.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class GetPipelinesUseCase {
  constructor(private readonly pipelineDashboardRepository: PipelineRepository) {}

  public async execute(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.pipelineDashboardRepository.getByProjectId(projectId, pagination);
  }
}

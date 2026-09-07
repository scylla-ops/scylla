import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

export class GetPipelinesMetadataUseCase {
  constructor(private readonly pipelineDashboardRepository: PipelineRepository) {}

  public async execute(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<PipelineMetadata>>> {
    return this.pipelineDashboardRepository.getMetadataByProjectId(projectId, pagination);
  }
}

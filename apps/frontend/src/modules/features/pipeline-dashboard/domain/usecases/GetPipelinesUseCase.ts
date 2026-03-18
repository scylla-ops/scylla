import type { PipelineDashboardRepository } from '@/modules/features/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';

export class GetPipelinesUseCase {
  constructor(private readonly pipelineDashboardRepository: PipelineDashboardRepository) {}

  public async execute(): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.pipelineDashboardRepository.getAll();
  }
}

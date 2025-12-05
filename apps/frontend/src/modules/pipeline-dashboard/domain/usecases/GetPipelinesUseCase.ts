import type { PipelineDashboardRepository } from '@/modules/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';

export class GetPipelinesUseCase {
  constructor(private readonly pipelineDashboardRepository: PipelineDashboardRepository) {}

  public async execute(): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.pipelineDashboardRepository.getPipelines();
  }
}

import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult';
import type { PipelineDashboardRepository } from '../repository/PipelineDashboardRepository';

export class RunPipeline {
  constructor(private readonly repository: PipelineDashboardRepository) {}
  public execute(pipelineId: string): Promise<ScyllaResult<void>> {
    return this.repository.run(pipelineId);
  }
}

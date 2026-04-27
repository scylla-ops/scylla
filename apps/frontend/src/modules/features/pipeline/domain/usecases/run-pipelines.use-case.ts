import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PipelineRepository } from '../repository/pipeline.repository.ts';

export class RunPipelinesUseCase {
  constructor(private readonly repository: PipelineRepository) {}
  public execute(pipelineId: string): Promise<ScyllaResult<void>> {
    return this.repository.run(pipelineId);
  }
}

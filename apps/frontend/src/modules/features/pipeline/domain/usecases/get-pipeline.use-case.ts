import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Pipeline } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export class GetPipelineUseCase {
  constructor(private readonly repository: PipelineRepository) {}

  public execute(pipelineId: string): Promise<ScyllaResult<Pipeline>> {
    return this.repository.getById(pipelineId);
  }
}

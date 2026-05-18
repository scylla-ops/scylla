import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { Pipeline } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export class CreatePipelineUseCase {
  constructor(private readonly _repository: PipelineRepository) {}

  public async execute(pipeline: Omit<Pipeline, 'id'>): Promise<ScyllaResult<void>> {
    return this._repository.create(pipeline);
  }
}

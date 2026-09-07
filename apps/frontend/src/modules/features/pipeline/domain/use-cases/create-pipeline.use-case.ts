import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';

export class CreatePipelineUseCase {
  constructor(private readonly _repository: PipelineRepository) {}

  public async execute(pipeline: Omit<PipelineEntity, 'id'>): Promise<ScyllaResult<void>> {
    return this._repository.create(pipeline);
  }
}

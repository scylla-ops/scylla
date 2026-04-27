import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PipelineRepository } from '@/modules/features/pipeline-dashboard/domain/repository/pipeline.repository.ts';

export class CreatePipelineUseCase {
  constructor(private readonly _repository: PipelineRepository) {}

  public async execute(content: string): Promise<ScyllaResult<void>> {
    return this._repository.create(content);
  }
}

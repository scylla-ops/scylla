import type { PipelineCreationRepository } from '@/modules/features/pipeline-creation/domain/repository/PipelineCreationRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export class CreatePipelineUsecase {
  constructor(private readonly _repository: PipelineCreationRepository) {}

  public async execute(content: string): Promise<ScyllaResult<void>> {
    return this._repository.createPipeline(content);
  }
}

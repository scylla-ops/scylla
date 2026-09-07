import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';

export class DeletePipelineUseCase {
  constructor(private readonly _repository: PipelineRepository) {}

  public execute = (id: string) => this._repository.deleteById(id);
}

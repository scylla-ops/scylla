import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type {
  Pipeline,
  PipelineStep,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class UpdatePipelineUseCase {
  constructor(private readonly repository: PipelineRepository) {}

  public async execute(
    id: string,
    steps: PipelineStep[],
    name?: string,
  ): Promise<ScyllaResult<Pipeline>> {
    return this.repository.edit(id, steps, name);
  }
}

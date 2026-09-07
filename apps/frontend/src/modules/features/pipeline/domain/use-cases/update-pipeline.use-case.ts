import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class UpdatePipelineUseCase {
  constructor(private readonly repository: PipelineRepository) {}

  public async execute(
    id: string,
    steps: PipelineStep[],
    name?: string,
  ): Promise<ScyllaResult<PipelineEntity>> {
    return this.repository.edit(id, steps, name);
  }
}

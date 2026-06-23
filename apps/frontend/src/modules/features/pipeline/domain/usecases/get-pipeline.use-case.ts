import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';

export class GetPipelineUseCase {
  constructor(private readonly repository: PipelineRepository) {}

  public execute(pipelineId: string): Promise<ScyllaResult<PipelineEntity>> {
    return this.repository.getById(pipelineId);
  }
}

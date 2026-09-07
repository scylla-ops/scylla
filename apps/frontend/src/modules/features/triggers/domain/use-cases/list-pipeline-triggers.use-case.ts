import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: list a pipeline's triggers. */
export class ListPipelineTriggersUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(pipelineId: string): Promise<ScyllaResult<TriggerEntity[]>> {
    return this.repository.listByPipelineId(pipelineId);
  }
}

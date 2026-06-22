import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedTrigger,
  TriggerDraft,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: create a trigger on a pipeline. */
export class CreateTriggerUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(pipelineId: string, draft: TriggerDraft): Promise<ScyllaResult<CreatedTrigger>> {
    return this.repository.create(pipelineId, draft);
  }
}

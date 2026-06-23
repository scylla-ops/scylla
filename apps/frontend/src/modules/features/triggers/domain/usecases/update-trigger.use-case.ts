import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: update a trigger's editable fields (name, source spec, inputs). */
export class UpdateTriggerUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(triggerId: string, draft: TriggerDraft): Promise<ScyllaResult<TriggerEntity>> {
    return this.repository.update(triggerId, draft);
  }
}

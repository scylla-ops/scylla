import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: enable/disable a trigger without deleting it. */
export class SetTriggerEnabledUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(triggerId: string, enabled: boolean): Promise<ScyllaResult<TriggerEntity>> {
    return this.repository.setEnabled(triggerId, enabled);
  }
}

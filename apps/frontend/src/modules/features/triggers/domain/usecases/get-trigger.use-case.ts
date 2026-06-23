import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: fetch a single trigger by id. */
export class GetTriggerUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(triggerId: string): Promise<ScyllaResult<TriggerEntity>> {
    return this.repository.getById(triggerId);
  }
}

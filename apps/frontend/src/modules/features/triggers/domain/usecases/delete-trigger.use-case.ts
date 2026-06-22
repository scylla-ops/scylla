import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: delete a trigger. */
export class DeleteTriggerUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(triggerId: string): Promise<ScyllaResult<boolean>> {
    return this.repository.deleteById(triggerId);
  }
}

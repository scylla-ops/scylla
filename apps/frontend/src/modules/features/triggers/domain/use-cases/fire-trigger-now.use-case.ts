import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: fire a trigger immediately (bypassing schedule/signature) for testing. */
export class FireTriggerNowUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  /** Resolves to the id of the job the fire minted. */
  public execute(triggerId: string): Promise<ScyllaResult<string>> {
    return this.repository.fireNow(triggerId);
  }
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: fire a trigger immediately (bypassing schedule/signature) for testing. */
export class FireTriggerNowUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(triggerId: string): Promise<ScyllaResult<Job>> {
    return this.repository.fireNow(triggerId);
  }
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';

/** Use case: fire a trigger immediately (bypassing schedule/signature) for testing. */
export class FireTriggerNowUseCase {
  constructor(private readonly repository: TriggersRepository) {}

  public execute(triggerId: string): Promise<ScyllaResult<JobEntity>> {
    return this.repository.fireNow(triggerId);
  }
}

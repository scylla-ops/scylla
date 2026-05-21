import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CreatedWorker } from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/** Use case: create a worker. Returns the one-time secret. */
export class CreateWorkerUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  execute(organizationId: string, name: string): Promise<ScyllaResult<CreatedWorker>> {
    return this.repository.createWorker(organizationId, name);
  }
}

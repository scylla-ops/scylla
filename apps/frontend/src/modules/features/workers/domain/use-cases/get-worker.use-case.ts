import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Worker } from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/** Use case: fetch a single worker by id. */
export class GetWorkerUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  execute(workerId: string): Promise<ScyllaResult<Worker>> {
    return this.repository.getWorker(workerId);
  }
}

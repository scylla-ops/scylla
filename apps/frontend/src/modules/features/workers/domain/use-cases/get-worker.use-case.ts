import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Worker } from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/**
 * Use case: Get a single worker by ID
 */
export class GetWorkerUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  async execute(workerId: string): Promise<ScyllaResult<Worker>> {
    return this.repository.getWorker(workerId);
  }
}

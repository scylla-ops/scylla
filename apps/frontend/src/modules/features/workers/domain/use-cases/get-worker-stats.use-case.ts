import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { WorkerStats } from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/** Use case: fetch the run stats of a worker. */
export class GetWorkerStatsUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  execute(workerId: string): Promise<ScyllaResult<WorkerStats>> {
    return this.repository.getWorkerStats(workerId);
  }
}

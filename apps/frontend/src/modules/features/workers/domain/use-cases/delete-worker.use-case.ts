import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/** Use case: delete a worker (revokes its grants, disconnects its agent). */
export class DeleteWorkerUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  execute(workerId: string): Promise<ScyllaResult<boolean>> {
    return this.repository.deleteWorker(workerId);
  }
}

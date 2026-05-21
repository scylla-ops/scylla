import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Worker } from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/** Use case: list the workers of an organization. */
export class GetWorkersUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  execute(organizationId: string): Promise<ScyllaResult<Worker[]>> {
    return this.repository.listWorkers(organizationId);
  }
}

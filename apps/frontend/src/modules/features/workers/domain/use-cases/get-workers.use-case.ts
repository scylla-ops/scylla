import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { WorkersListResponse } from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';

/**
 * Use case: List all workers
 */
export class GetWorkersUseCase {
  constructor(private readonly repository: WorkersRepository) {}

  async execute(pagination?: {
    page?: number;
    pageSize?: number;
  }): Promise<ScyllaResult<WorkersListResponse>> {
    return this.repository.listWorkers(pagination);
  }
}

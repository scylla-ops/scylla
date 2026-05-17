import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';
import type { WorkersRemoteDataSource } from '@/modules/features/workers/infrastructure/data/workers-remote.data-source.impl.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  WorkersListResponse,
  Worker,
} from '@/modules/features/workers/domain/models/worker.model.ts';

/**
 * Implementation of WorkersRepository using remote data source
 */
export class DefaultWorkersRepository implements WorkersRepository {
  constructor(private remoteDataSource: WorkersRemoteDataSource) {}

  listWorkers(pagination?: {
    page?: number;
    pageSize?: number;
  }): Promise<ScyllaResult<WorkersListResponse>> {
    return this.remoteDataSource.listWorkers(pagination);
  }

  getWorker(workerId: string): Promise<ScyllaResult<Worker>> {
    return this.remoteDataSource.getWorker(workerId);
  }
}

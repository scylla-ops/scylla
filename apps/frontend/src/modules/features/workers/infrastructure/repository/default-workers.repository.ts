import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';
import type { WorkersRemoteDataSource } from '@/modules/features/workers/infrastructure/data/workers-remote.data-source.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedWorker,
  Worker,
  WorkerStats,
} from '@/modules/features/workers/domain/models/worker.model.ts';

/** WorkersRepository backed by the remote (gRPC) data source. */
export class DefaultWorkersRepository implements WorkersRepository {
  constructor(private remoteDataSource: WorkersRemoteDataSource) {}

  listWorkers(organizationId: string): Promise<ScyllaResult<Worker[]>> {
    return this.remoteDataSource.listWorkers(organizationId);
  }

  getWorker(workerId: string): Promise<ScyllaResult<Worker>> {
    return this.remoteDataSource.getWorker(workerId);
  }

  getWorkerStats(workerId: string): Promise<ScyllaResult<WorkerStats>> {
    return this.remoteDataSource.getWorkerStats(workerId);
  }

  createWorker(organizationId: string, name: string): Promise<ScyllaResult<CreatedWorker>> {
    return this.remoteDataSource.createWorker(organizationId, name);
  }

  deleteWorker(workerId: string): Promise<ScyllaResult<boolean>> {
    return this.remoteDataSource.deleteWorker(workerId);
  }
}

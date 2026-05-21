import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { WorkerAdminServiceClient } from '@/generated/worker_admin.client.ts';
import type {
  CreatedWorker,
  Worker,
  WorkerStats,
} from '@/modules/features/workers/domain/models/worker.model.ts';
import type { WorkersRepository } from '@/modules/features/workers/domain/repository/workers.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { GrpcWorkerMapper } from './grpc-worker.mapper.ts';

/** Remote data source for workers over gRPC. Shares the repository contract. */
export type WorkersRemoteDataSource = WorkersRepository;

export class WorkersRemoteDataSourceImpl implements WorkersRemoteDataSource {
  constructor(private grpcTransport: CoreGrpcTransport) {}

  listWorkers(organizationId: string): Promise<ScyllaResult<Worker[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new WorkerAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.listWorkers({ organizationId }).response;
      return response.workers.map(worker => GrpcWorkerMapper.toDomain(worker));
    }, 'Failed to list workers');
  }

  getWorker(workerId: string): Promise<ScyllaResult<Worker>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new WorkerAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.getWorker({ id: workerId }).response;
      return GrpcWorkerMapper.toDomain(response);
    }, 'Failed to fetch worker');
  }

  getWorkerStats(workerId: string): Promise<ScyllaResult<WorkerStats>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new WorkerAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.getWorkerStats({ id: workerId }).response;
      return GrpcWorkerMapper.statsToDomain(response);
    }, 'Failed to fetch worker stats');
  }

  createWorker(organizationId: string, name: string): Promise<ScyllaResult<CreatedWorker>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new WorkerAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.createWorker({ organizationId, name }).response;
      if (!response.worker) throw new Error('CreateWorker returned no worker');
      return { worker: GrpcWorkerMapper.toDomain(response.worker), secret: response.secret };
    }, 'Failed to create worker');
  }

  deleteWorker(workerId: string): Promise<ScyllaResult<boolean>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new WorkerAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.deleteWorker({ id: workerId }).response;
      return response.deleted;
    }, 'Failed to delete worker');
  }
}

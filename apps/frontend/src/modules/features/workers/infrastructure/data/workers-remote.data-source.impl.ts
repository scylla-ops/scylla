import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { AgentServiceClient } from '@/generated/agent.client.ts';
import type {
  WorkersListResponse,
  Worker,
} from '@/modules/features/workers/domain/models/worker.model.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { GrpcWorkerMapper } from './grpc-worker.mapper.ts';

/**
 * Data source interface for remote worker operations
 */
export interface WorkersRemoteDataSource {
  listWorkers(pagination?: {
    page?: number;
    pageSize?: number;
  }): Promise<ScyllaResult<WorkersListResponse>>;
  getWorker(workerId: string): Promise<ScyllaResult<Worker>>;
}

/**
 * Implementation of workers data source using gRPC
 */
export class WorkersRemoteDataSourceImpl implements WorkersRemoteDataSource {
  constructor(private grpcTransport: CoreGrpcTransport) {}

  async listWorkers(pagination?: {
    page?: number;
    pageSize?: number;
  }): Promise<ScyllaResult<WorkersListResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentServiceClient(this.grpcTransport.getTransport());
      const requestPagination = pagination
        ? {
            page: pagination.page ?? 1,
            pageSize: pagination.pageSize ?? 20,
          }
        : undefined;
      const response = await client.listAgents({ pagination: requestPagination }).response;
      return GrpcWorkerMapper.toDomainList(response);
    }, 'Failed to list workers');
  }

  async getWorker(workerId: string): Promise<ScyllaResult<Worker>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentServiceClient(this.grpcTransport.getTransport());
      const response = await client.getAgent({ agentId: workerId }).response;
      return GrpcWorkerMapper.toDomain(response);
    }, 'Failed to fetch worker');
  }
}

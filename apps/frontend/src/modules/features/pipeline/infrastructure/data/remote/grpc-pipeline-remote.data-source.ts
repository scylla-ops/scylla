import type { PipelineRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CreatePipelineRequest, ListPipelinesResponse } from '@/generated/pipeline.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import {
  DEFAULT_PAGE_SIZE,
  type PaginationParams,
} from '@shared/domain/models/pagination.model.ts';

export class GrpcPipelineRemoteDataSource implements PipelineRemoteDataSource {
  private readonly _pipelineClient: PipelineServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async deleteById(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.deletePipeline({ pipelineId: id });
    }, 'Error deleting pipeline');
  }

  public async create(content: string): Promise<ScyllaResult<void>> {
    return await ScyllaResult.tryAsync<void>(async () => {
      const request: CreatePipelineRequest = JSON.parse(content);

      await this._pipelineClient.createPipeline(request);
    }, 'Failed to create pipeline.');
  }

  public async getByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>> {
    return ScyllaResult.tryAsync<ListPipelinesResponse>(
      async () =>
        (
          await this._pipelineClient.listProjectPipelines({
            projectId,
            pagination: pagination ?? { page: 1, pageSize: DEFAULT_PAGE_SIZE },
          })
        ).response,
      'Error getting pipelines',
    );
  }

  public async run(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.runPipeline({ pipelineId: id });
    }, 'Error running pipeline');
  }
}

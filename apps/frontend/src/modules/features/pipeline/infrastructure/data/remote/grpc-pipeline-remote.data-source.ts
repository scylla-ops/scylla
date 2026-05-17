import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatePipelineRequest,
  ListPipelinesResponse,
  PipelineNode,
  PipelineResponse,
} from '@/generated/pipeline.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import {
  DEFAULT_PAGE_SIZE,
  type PaginationParams,
} from '@shared/domain/models/pagination.model.ts';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';

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

  public async create(request: CreatePipelineRequest): Promise<ScyllaResult<void>> {
    return await ScyllaResult.tryAsync<void>(async () => {

      await this._pipelineClient.createPipeline(request);
    }, 'Failed to create pipeline.');
  }

  /** Return list of summary of pipelines **/
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

  public async getById(id: string): Promise<ScyllaResult<PipelineResponse>> {
    return ScyllaResult.tryAsync<PipelineResponse>(async () => {
      return (await this._pipelineClient.getPipeline({ pipelineId: id })).response;
    }, 'Error getting pipeline');
  }

  public async run(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.runPipeline({ pipelineId: id });
    }, 'Error running pipeline');
  }

  public async update(id: string, nodes: PipelineNode[], name?: string) {
    return ScyllaResult.tryAsync<PipelineResponse>(async () => {
      return await this._pipelineClient.updatePipeline({
        pipelineId: id,
        nodes: nodes,
        name: name,
      }).response;
    }, 'Failed to edit pipeline.');
  }
}

import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatePipelineRequest,
  ListProjectPipelinesResponse,
  Pipeline,
  PipelineNode,
} from '@/generated/scylla/pipeline/v1/pipeline.ts';
import { PipelineServiceClient } from '@/generated/scylla/pipeline/v1/pipeline.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import {
  DEFAULT_PAGE_SIZE,
  type PaginationParams,
} from '@shared/domain/structs/pagination.struct.ts';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

/**
 * Every RPC now answers with a wrapper message holding the entity in field 1.
 * The field is optional on the wire, so an old or broken server could omit it —
 * fail loudly here rather than let `undefined` reach a mapper.
 */
function requirePipeline(pipeline: Pipeline | undefined): Pipeline {
  if (!pipeline) throw new Error('Server response carried no pipeline');
  return pipeline;
}

export class GrpcPipelineRemoteDataSource implements PipelineRemoteDataSource {
  private readonly _pipelineClient: PipelineServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async deleteById(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.deletePipeline({ pipelineId: wrapId(id) });
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
  ): Promise<ScyllaResult<ListProjectPipelinesResponse>> {
    return ScyllaResult.tryAsync<ListProjectPipelinesResponse>(
      async () =>
        (
          await this._pipelineClient.listProjectPipelines({
            projectId: wrapId(projectId),
            pagination: pagination ?? { page: 1, pageSize: DEFAULT_PAGE_SIZE },
          })
        ).response,
      'Error getting pipelines',
    );
  }

  public async getById(id: string): Promise<ScyllaResult<Pipeline>> {
    return ScyllaResult.tryAsync<Pipeline>(async () => {
      const { response } = await this._pipelineClient.getPipeline({ pipelineId: wrapId(id) });
      return requirePipeline(response.pipeline);
    }, 'Error getting pipeline');
  }

  /**
   * Enqueue a run. The server answers with the id of the job it minted; nothing
   * downstream needs it today (the caller refetches the pipeline's job list),
   * so it is dropped here.
   */
  public async run(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.runPipeline({ pipelineId: wrapId(id) });
    }, 'Error running pipeline');
  }

  public async update(id: string, nodes: PipelineNode[], name?: string) {
    return ScyllaResult.tryAsync<Pipeline>(async () => {
      const { response } = await this._pipelineClient.updatePipeline({
        pipelineId: wrapId(id),
        nodes: nodes,
        name: name,
      });
      return requirePipeline(response.pipeline);
    }, 'Failed to edit pipeline.');
  }
}

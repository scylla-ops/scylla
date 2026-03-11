import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { PipelineCreationRemoteStore } from '@/modules/pipeline-creation/repository/stores/PipelineCreationRemoteStore.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import type { CreatePipelineRequest, PipelineResponse } from '@/generated/pipeline.ts';

export class PipelineCreationRemoteStoreImpl implements PipelineCreationRemoteStore {
  private readonly _pipelineClient: PipelineServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async createPipeline(content: string): Promise<ScyllaResult<void>> {
    return await ScyllaResult.tryAsync<void>(async () => {
      const request: CreatePipelineRequest = JSON.parse(content);
      await this._pipelineClient.createPipeline(request);
    }, 'Failed to create pipeline.');
  }

  public async getPipelineById(id: string): Promise<ScyllaResult<PipelineResponse>> {
    return ScyllaResult.tryAsync(
      async () => (await this._pipelineClient.getPipeline({ pipelineId: id })).response,
      'Failed to get pipeline by id.',
    );
  }
}

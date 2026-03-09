import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { PipelineCreationRemoteStore } from '@/modules/pipeline-creation/repository/stores/PipelineCreationRemoteStore.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import { CreatePipelineRequest, PipelineResponse } from '@/generated/pipeline.ts';

export class PipelineCreationRemoteStoreImpl implements PipelineCreationRemoteStore {
  private readonly _pipelineClient: PipelineServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async createPipeline(content: string): Promise<ScyllaResult<void>> {
    try {
      //await this._pipelineClient.createPipeline({ content: content });
      return { ok: true, value: undefined };
    } catch (error) {
      return { ok: false, error: { message: `Failed to create pipeline.` + error } };
    }
  }
}

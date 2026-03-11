import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { PipelineCreationRemoteStore } from '@/modules/pipeline-creation/repository/stores/PipelineCreationRemoteStore.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import type { PipelineResponse } from '@/generated/pipeline.ts';

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

  public async getPipelineById(id: string): Promise<ScyllaResult<PipelineResponse>> {
    try {
      const pipeline = await this._pipelineClient.getPipeline({ pipelineId: id });
      return { ok: true, value: pipeline.response };
    } catch (error) {
      return { ok: false, error: { message: `Failed to fetch pipeline for id: ${id}` + error } };
    }
  }
}

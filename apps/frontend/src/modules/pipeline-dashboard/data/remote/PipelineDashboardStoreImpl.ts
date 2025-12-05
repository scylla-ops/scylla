import type { PipelineDashboardStore } from '@/modules/pipeline-dashboard/repository/store/PipelineDashboardStore.ts';
import { PipelineClient } from '@/generated/pipeline.client.ts';
import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListPipelinesResponse, PipelineResponse } from '@/generated/pipeline.ts';

export class PipelineDashboardStoreImpl implements PipelineDashboardStore {
  private readonly _pipelineClient: PipelineClient;

  constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineClient(transport.getTransport());
  }

  async getPipelineStatsById(id: string): Promise<ScyllaResult<PipelineResponse>> {
    try {
      const { response } = await this._pipelineClient.getPipeline({ pipelineId: id });
      return { ok: true, value: response };
    } catch (error) {
      return { ok: false, error: { message: `Failed to fetch pipeline for id: ${id}` + error } };
    }
  }

  async getPipelines(): Promise<ScyllaResult<ListPipelinesResponse>> {
    try {
      const { response } = await this._pipelineClient.listPipelines({
        pagination: {
          page: 1,
          pageSize: 10,
        },
      });
      return { ok: true, value: response };
    } catch (error) {
      return { ok: false, error: { message: `Failed to fetch pipelines.` + error } };
    }
  }
}

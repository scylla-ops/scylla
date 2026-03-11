import type { PipelineDashboardStore } from '@/modules/pipeline-dashboard/repository/store/PipelineDashboardStore.ts';
import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';

export class PipelineDashboardStoreImpl implements PipelineDashboardStore {
  private readonly _pipelineClient: PipelineServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  async getPipelines(): Promise<ScyllaResult<ListPipelinesResponse>> {
    const pagination = {
      page: 1,
      pageSize: 10,
    };

    return ScyllaResult.tryAsync<ListPipelinesResponse>(
      async () => (await this._pipelineClient.listPipelines({ pagination })).response,
      'Error getting pipelines',
    );
  }
}

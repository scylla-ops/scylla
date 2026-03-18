import type { PipelineDashboardRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/PipelineDashboardRemoteDataSource.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';

export class PipelineDashboardRemoteDataSourceImpl implements PipelineDashboardRemoteDataSource {
  private readonly _pipelineClient: PipelineServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async getAll(): Promise<ScyllaResult<ListPipelinesResponse>> {
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

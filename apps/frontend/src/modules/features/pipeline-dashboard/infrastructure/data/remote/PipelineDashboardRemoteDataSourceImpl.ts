import type { PipelineDashboardRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/PipelineDashboardRemoteDataSource.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import type { GrpcTransport } from '@core/infrastructure/grpc/GrpcTransport.ts';

export class PipelineDashboardRemoteDataSourceImpl implements PipelineDashboardRemoteDataSource {
  private readonly _pipelineClient: PipelineServiceClient;

  public constructor(transport: GrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async deleteById(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.deletePipeline({ pipelineId: id });
    }, 'Error deleting pipeline');
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

  public async run(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.runPipeline({ pipelineId: id });
    }, 'Error running pipeline');
  }
}

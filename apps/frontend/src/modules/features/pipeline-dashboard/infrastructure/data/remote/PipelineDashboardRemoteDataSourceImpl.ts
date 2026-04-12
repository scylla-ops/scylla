import type { PipelineDashboardRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/PipelineDashboardRemoteDataSource.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import { PipelineServiceClient } from '@/generated/pipeline.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';
import { DEFAULT_PAGE_SIZE } from '@/modules/shared/domain/types/Pagination.ts';

export class PipelineDashboardRemoteDataSourceImpl implements PipelineDashboardRemoteDataSource {
  private readonly _pipelineClient: PipelineServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._pipelineClient = new PipelineServiceClient(transport.getTransport());
  }

  public async deleteById(id: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._pipelineClient.deletePipeline({ pipelineId: id });
    }, 'Error deleting pipeline');
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

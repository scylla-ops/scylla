import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/JobsRemoteDataSource.ts';
import type { ListJobsResponse, JobResponse } from '@/generated/job.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { PaginationRequest } from '@/generated/common.ts';
import { ScyllaResult as Result } from '@/modules/shared/utils/ScyllaResult.ts';
import { JobServiceClient } from '@/generated/job.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';

export class JobsRemoteDataSourceImpl implements JobsRemoteDataSource {
  private readonly _jobClient: JobServiceClient;

  constructor(grpcTransport: CoreGrpcTransport) {
    this._jobClient = new JobServiceClient(grpcTransport.getTransport());
  }

  public async getByPipelineId(
    pipelineId: string,
    pagination?: PaginationRequest,
  ): Promise<ScyllaResult<ListJobsResponse>> {
    return Result.tryAsync<ListJobsResponse>(
      async () => (await this._jobClient.listPipelineJobs({ pipelineId, pagination })).response,
      'Error fetching pipeline jobs',
    );
  }

  public async getById(jobId: string): Promise<ScyllaResult<JobResponse>> {
    return Result.tryAsync<JobResponse>(
      async () => (await this._jobClient.getJob({ jobId })).response,
      'Error fetching job',
    );
  }

  public async deleteById(jobId: string): Promise<ScyllaResult<void>> {
    return Result.tryAsync<void>(async () => {
      await this._jobClient.deleteJob({ jobId });
    }, 'Error deleting job');
  }
}

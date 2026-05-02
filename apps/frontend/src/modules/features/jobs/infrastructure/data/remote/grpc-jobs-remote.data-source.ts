import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import type {
  ListJobsResponse,
  JobResponse,
  ListJobLogsResponse,
  JobLogEntry,
} from '@/generated/job.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';
import type { JobLogsTailHandle } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import { ScyllaResult as Result } from '@shared/utils/scylla-result.ts';
import { JobServiceClient } from '@/generated/job.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';

export class GrpcJobsRemoteDataSource implements JobsRemoteDataSource {
  private readonly _jobClient: JobServiceClient;

  constructor(grpcTransport: CoreGrpcTransport) {
    this._jobClient = new JobServiceClient(grpcTransport.getTransport());
  }

  public async getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
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

  public async getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobLogsResponse>> {
    return Result.tryAsync<ListJobLogsResponse>(
      async () =>
        (await this._jobClient.listJobLogs({ jobId, nodeId, pagination })).response,
      'Error fetching job logs',
    );
  }

  public tailLogs(jobId: string, nodeId?: string): JobLogsTailHandle {
    const abortController = new AbortController();
    const call = this._jobClient.tailJobLogs(
      { jobId, nodeId },
      { abort: abortController.signal },
    );
    const responses: AsyncIterable<JobLogEntry> = {
      [Symbol.asyncIterator]: async function* () {
        for await (const evt of call.responses) {
          if (evt.log) {
            yield evt.log;
          }
        }
      },
    };
    return {
      responses,
      cancel: () => abortController.abort(),
    };
  }
}

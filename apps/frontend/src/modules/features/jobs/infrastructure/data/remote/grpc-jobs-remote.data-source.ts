import type {
  JobsRemoteDataSource,
  JobLogsTailHandleRepo,
} from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import type {
  ListJobsResponse,
  JobResponse,
  ListJobLogsResponse,
  JobLogEntry,
} from '@/generated/job.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import { ScyllaResult as Result } from '@shared/utils/scylla-result.ts';
import { JobServiceClient } from '@/generated/job.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';

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
    return Result.tryAsync<ListJobLogsResponse>(async () => {
      const rep = (await this._jobClient.listJobLogs({ jobId, nodeId, pagination })).response;
      console.log('[gRPC getLogs] Response:', rep);
      return rep;
    }, 'Error fetching job logs');
  }

  public tailLogs(jobId: string, nodeId?: string): ScyllaResult<JobLogsTailHandleRepo> {
    const abortController = new AbortController();

    return ScyllaResult.try<JobLogsTailHandleRepo>(() => {
      const call = this._jobClient.tailJobLogs(
        { jobId, nodeId },
        { abort: abortController.signal },
      );

      const responses: AsyncIterable<ScyllaResult<JobLogEntry>> = {
        [Symbol.asyncIterator]: async function* () {
          try {
            for await (const evt of call.responses) {
              if (evt.log) {
                yield ScyllaResult.success(evt.log);
              }
            }
          } catch (err) {
            yield ScyllaResult.error(new ScyllaError('Error tailing job logs', { cause: err }));
          }
        },
      };

      return {
        responses,
        cancel: () => abortController.abort(),
      };
    }, 'Error tailing job logs');
  }
}

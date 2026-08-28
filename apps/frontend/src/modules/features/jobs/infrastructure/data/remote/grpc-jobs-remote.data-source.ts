import type {
  JobsRemoteDataSource,
  JobLogsTailHandleRepo,
} from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import type {
  Job,
  ListPipelineJobsResponse,
  ListJobLogsResponse,
  JobLogEntry,
} from '@/generated/scylla/job/v1/job.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import { ScyllaResult as Result } from '@shared/utils/scylla-result.ts';
import { JobServiceClient } from '@/generated/scylla/job/v1/job.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { wrapId, wrapIdOpt } from '@shared/infrastructure/grpc/wrappers.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export class GrpcJobsRemoteDataSource implements JobsRemoteDataSource {
  private readonly _jobClient: JobServiceClient;

  constructor(grpcTransport: CoreGrpcTransport) {
    this._jobClient = new JobServiceClient(grpcTransport.getTransport());
  }

  public async getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelineJobsResponse>> {
    return Result.tryAsync<ListPipelineJobsResponse>(
      async () =>
        (await this._jobClient.listPipelineJobs({ pipelineId: wrapId(pipelineId), pagination }))
          .response,
      'Error fetching pipeline jobs',
    );
  }

  public async getById(jobId: string): Promise<ScyllaResult<Job>> {
    return Result.tryAsync<Job>(async () => {
      // `GetJobResponse` wraps the entity; unwrap here so mappers keep seeing a `Job`.
      const { job } = (await this._jobClient.getJob({ jobId: wrapId(jobId) })).response;
      if (!job) throw new ScyllaError('Job missing from GetJobResponse');
      return job;
    }, 'Error fetching job');
  }

  public async deleteById(jobId: string): Promise<ScyllaResult<void>> {
    return Result.tryAsync<void>(async () => {
      await this._jobClient.deleteJob({ jobId: wrapId(jobId) });
    }, 'Error deleting job');
  }

  public async getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobLogsResponse>> {
    return Result.tryAsync<ListJobLogsResponse>(async () => {
      return (
        await this._jobClient.listJobLogs({
          jobId: wrapId(jobId),
          nodeId: wrapIdOpt(nodeId),
          pagination,
        })
      ).response;
    }, 'Error fetching job logs');
  }

  public tailLogs(jobId: string, nodeId?: string): ScyllaResult<JobLogsTailHandleRepo> {
    const abortController = new AbortController();

    return ScyllaResult.try<JobLogsTailHandleRepo>(() => {
      const call = this._jobClient.tailJobLogs(
        { jobId: wrapId(jobId), nodeId: wrapIdOpt(nodeId) },
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

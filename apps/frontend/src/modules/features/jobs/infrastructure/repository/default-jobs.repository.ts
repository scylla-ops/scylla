import type {
  JobsRepository,
  JobLogsTailHandle,
} from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ListJobsResponse, JobResponse, ListJobLogsResponse } from '@/generated/job.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';

export class DefaultJobsRepository implements JobsRepository {
  constructor(private readonly remoteDataSource: JobsRemoteDataSource) {}

  public async getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobsResponse>> {
    return this.remoteDataSource.getByPipelineId(pipelineId, pagination);
  }

  public async getById(jobId: string): Promise<ScyllaResult<JobResponse>> {
    return this.remoteDataSource.getById(jobId);
  }

  public async deleteById(jobId: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.deleteById(jobId);
  }

  public async getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobLogsResponse>> {
    return this.remoteDataSource.getLogs(jobId, nodeId, pagination);
  }

  public tailLogs(jobId: string, nodeId?: string): JobLogsTailHandle {
    return this.remoteDataSource.tailLogs(jobId, nodeId);
  }
}

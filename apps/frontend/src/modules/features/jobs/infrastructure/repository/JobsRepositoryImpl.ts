import type { JobsRepository } from '@/modules/features/jobs/domain/repository/JobsRepository.ts';
import type { ListJobsResponse, JobResponse } from '@/generated/job.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/JobsRemoteDataSource.ts';

export class JobsRepositoryImpl implements JobsRepository {
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
}

import type { ListJobsResponse, JobResponse } from '@/generated/job.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { PaginationRequest } from '@/generated/common.ts';

export interface JobsRemoteDataSource {
  getByPipelineId(
    pipelineId: string,
    pagination?: PaginationRequest,
  ): Promise<ScyllaResult<ListJobsResponse>>;
  getById(jobId: string): Promise<ScyllaResult<JobResponse>>;
  deleteById(jobId: string): Promise<ScyllaResult<void>>;
}


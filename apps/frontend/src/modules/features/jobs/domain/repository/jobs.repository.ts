import type { ListJobsResponse, JobResponse } from '@/generated/job.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export interface JobsRepository {
  getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobsResponse>>;
  getById(jobId: string): Promise<ScyllaResult<JobResponse>>;
  deleteById(jobId: string): Promise<ScyllaResult<void>>;
}

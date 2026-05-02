import type {
  ListJobsResponse,
  JobResponse,
  ListJobLogsResponse,
} from '@/generated/job.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';
import type { JobLogsTailHandle } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';

export interface JobsRemoteDataSource {
  getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobsResponse>>;
  getById(jobId: string): Promise<ScyllaResult<JobResponse>>;
  deleteById(jobId: string): Promise<ScyllaResult<void>>;
  getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListJobLogsResponse>>;
  tailLogs(jobId: string, nodeId?: string): JobLogsTailHandle;
}

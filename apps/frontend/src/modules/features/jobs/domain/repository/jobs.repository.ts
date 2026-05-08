import type {
  ListJobsResponse,
  JobResponse,
  ListJobLogsResponse,
  JobLogEntry,
} from '@/generated/job.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export interface JobLogsTailHandle {
  responses: AsyncIterable<JobLogEntry>;
  cancel: () => void;
}

export interface JobsRepository {
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

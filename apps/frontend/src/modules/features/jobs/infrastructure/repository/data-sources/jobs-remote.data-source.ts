import type {
  ListJobsResponse,
  JobResponse,
  ListJobLogsResponse,
  JobLogEntry,
} from '@/generated/job.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';

//tood: move this
export interface JobLogsTailHandleRepo {
  responses: AsyncIterable<ScyllaResult<JobLogEntry>>;
  cancel: () => void;
}

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
  tailLogs(jobId: string, nodeId?: string): ScyllaResult<JobLogsTailHandleRepo>;
}

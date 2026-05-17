import type { Job, JobLog, JobLogStream } from '@/modules/features/jobs/domain/models/job.model.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';

export interface JobsRepository {
  getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<Job>>>;
  getById(jobId: string): Promise<ScyllaResult<Job>>;
  deleteById(jobId: string): Promise<ScyllaResult<void>>;
  getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobLog>>>;
  tailLogs(jobId: string, nodeId?: string): ScyllaResult<JobLogStream>;
}

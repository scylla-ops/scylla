import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type { JobLog, JobLogStream } from '@/modules/features/jobs/domain/structs/job.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';

export interface JobsRepository {
  getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobEntity>>>;
  getById(jobId: string): Promise<ScyllaResult<JobEntity>>;
  deleteById(jobId: string): Promise<ScyllaResult<void>>;
  getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobLog>>>;
  tailLogs(jobId: string, nodeId?: string): ScyllaResult<JobLogStream>;
}

import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobLogStream } from '@/modules/features/jobs/domain/structs/job.struct.ts';

export class TailJobLogsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string, nodeId?: string): ScyllaResult<JobLogStream> {
    return this.repository.tailLogs(jobId, nodeId);
  }
}

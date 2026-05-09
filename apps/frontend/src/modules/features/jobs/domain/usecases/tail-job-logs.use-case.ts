import type {
  JobsRepository,
  JobLogsTailHandle,
} from '@/modules/features/jobs/domain/repository/jobs.repository.ts';

export class TailJobLogsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string, nodeId?: string): JobLogsTailHandle {
    return this.repository.tailLogs(jobId, nodeId);
  }
}

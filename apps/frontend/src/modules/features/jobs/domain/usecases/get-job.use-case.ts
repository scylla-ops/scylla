import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';

export class GetJobUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string): Promise<ScyllaResult<Job>> {
    return this.repository.getById(jobId);
  }
}

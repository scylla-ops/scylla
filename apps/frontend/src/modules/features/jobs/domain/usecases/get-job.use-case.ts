import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobResponse } from '@/generated/job.ts';

export class GetJobUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string): Promise<ScyllaResult<JobResponse>> {
    return this.repository.getById(jobId);
  }
}

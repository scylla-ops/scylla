import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

export class GetJobUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string): Promise<ScyllaResult<JobEntity>> {
    return this.repository.getById(jobId);
  }
}

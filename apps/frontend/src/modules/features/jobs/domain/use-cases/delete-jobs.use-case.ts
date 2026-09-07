import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class DeleteJobsUseCase {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string): Promise<ScyllaResult<void>> {
    return this.repository.deleteById(jobId);
  }
}

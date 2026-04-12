import type { JobsRepository } from '@/modules/features/jobs/domain/repository/JobsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export class DeleteJob {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string): Promise<ScyllaResult<void>> {
    return this.repository.deleteById(jobId);
  }
}


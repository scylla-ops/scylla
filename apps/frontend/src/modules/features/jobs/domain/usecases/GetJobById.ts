import type { JobsRepository } from '@/modules/features/jobs/domain/repository/JobsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { JobResponse } from '@/generated/job.ts';

export class GetJobById {
  constructor(private readonly repository: JobsRepository) {}

  public execute(jobId: string): Promise<ScyllaResult<JobResponse>> {
    return this.repository.getById(jobId);
  }
}


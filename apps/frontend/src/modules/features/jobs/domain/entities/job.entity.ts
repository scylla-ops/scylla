import type { JobNodeExecution } from '@/modules/features/jobs/domain/structs/job.struct.ts';

export interface JobEntity {
  id: string;
  pipelineId: string;
  status: string;
  nodeExecutions: JobNodeExecution[];
  createdAt: string;
  updatedAt: string;
  /** When execution actually began (a worker picked it up). Unset while pending. */
  startedAt?: string;
  /** When execution finished. Unset while pending/running. */
  finishedAt?: string;
}

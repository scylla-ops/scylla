import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedTrigger,
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

/** Repository interface for pipeline-scoped triggers. */
export interface TriggersRepository {
  listByPipelineId(pipelineId: string): Promise<ScyllaResult<TriggerEntity[]>>;
  getById(triggerId: string): Promise<ScyllaResult<TriggerEntity>>;
  create(pipelineId: string, draft: TriggerDraft): Promise<ScyllaResult<CreatedTrigger>>;
  update(triggerId: string, draft: TriggerDraft): Promise<ScyllaResult<TriggerEntity>>;
  deleteById(triggerId: string): Promise<ScyllaResult<boolean>>;
  setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<TriggerEntity>>;
  /** Fire immediately for testing — mints and dispatches a Job, like a real tir. */
  fireNow(triggerId: string): Promise<ScyllaResult<JobEntity>>;
}

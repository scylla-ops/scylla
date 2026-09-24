import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedTrigger,
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

export interface TriggersRepository {
  listByPipelineId(pipelineId: string): Promise<ScyllaResult<TriggerEntity[]>>;
  getById(triggerId: string): Promise<ScyllaResult<TriggerEntity>>;
  create(pipelineId: string, draft: TriggerDraft): Promise<ScyllaResult<CreatedTrigger>>;
  update(triggerId: string, draft: TriggerDraft): Promise<ScyllaResult<TriggerEntity>>;
  deleteById(triggerId: string): Promise<ScyllaResult<void>>;
  setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<TriggerEntity>>;
  /** Creates and dispatches a real job; returns its id. */
  fireNow(triggerId: string): Promise<ScyllaResult<string>>;
}

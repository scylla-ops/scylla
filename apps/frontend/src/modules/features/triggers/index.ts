/** What starts a pipeline without a human: schedules and webhooks. */
export type { TriggerEntity, TriggerDraft, CreatedTrigger } from './domain/entities/trigger.entity.ts';
export type { TriggerSource, TriggerInput } from './domain/structs/trigger-source.struct.ts';
export { TriggerKind } from './domain/structs/trigger-source.struct.ts';
export {
  usePipelineTriggers,
  TRIGGERS_QUERY_KEY,
} from './presentation/hooks/use-pipeline-triggers.ts';

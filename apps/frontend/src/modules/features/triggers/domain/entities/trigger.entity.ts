import type {
  TriggerInput,
  TriggerSource,
  TriggerSourceDraft,
} from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';

/**
 * Outcome of the last fire. `unknown` means the server reported an outcome arm
 * newer than this build — shown as such, never guessed at.
 */
export type TriggerFireResult =
  | { kind: 'succeeded' }
  | { kind: 'failed'; error: string }
  | { kind: 'unknown' };

/**
 * A stored, pipeline-scoped initiator that launches a run without a human
 * clicking "Run". Identity-bearing — the thing the feature owns.
 */
export interface TriggerEntity {
  id: string;
  pipelineId: string;
  name: string;
  source: TriggerSource;
  inputs: TriggerInput[];
  enabled: boolean;
  /** Cron only: next occurrence due (UTC, ISO string). */
  nextFireAt?: string;
  /** Last fire time, for observability. Unset before the first fire. */
  lastFiredAt?: string;
  /** Last fire result. Absent until the trigger has fired at least once. */
  lastResult?: TriggerFireResult;
  createdAt: string;
  updatedAt: string;
}

/** Editable fields for create/update — no identity, no server-managed fields. */
export interface TriggerDraft {
  name: string;
  source: TriggerSourceDraft;
  inputs: TriggerInput[];
}

/**
 * Returned only by create. Carries the one-time webhook signing secret in clear
 * — copy it now, it is never returned again. Absent for a cron trigger.
 */
export interface CreatedTrigger {
  trigger: TriggerEntity;
  webhookSecret?: string;
}

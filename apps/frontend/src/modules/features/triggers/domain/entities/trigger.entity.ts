import type {
  TriggerInput,
  TriggerSource,
  TriggerSourceDraft,
} from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';

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
  /** Last fire result: "" | "ok" | "error: ...". */
  lastStatus: string;
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
 * (empty for cron) — copy it now, it is never returned again.
 */
export interface CreatedTrigger {
  trigger: TriggerEntity;
  webhookSecret: string;
}

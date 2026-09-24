import type {
  TriggerInput,
  TriggerSource,
  TriggerSourceDraft,
} from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';

/** `unknown`: an outcome arm newer than this build. */
export type TriggerFireResult =
  | { kind: 'succeeded' }
  | { kind: 'failed'; error: string }
  | { kind: 'unknown' };

/** Launches a run of its pipeline without a human. */
export interface TriggerEntity {
  id: string;
  pipelineId: string;
  name: string;
  source: TriggerSource;
  inputs: TriggerInput[];
  enabled: boolean;
  /** Cron only, UTC. */
  nextFireAt?: string;
  lastFiredAt?: string;
  lastResult?: TriggerFireResult;
  createdAt: string;
  updatedAt: string;
}

export interface TriggerDraft {
  name: string;
  source: TriggerSourceDraft;
  inputs: TriggerInput[];
}

/** Returned only by create. The webhook secret is shown once and never returned again. */
export interface CreatedTrigger {
  trigger: TriggerEntity;
  webhookSecret?: string;
}

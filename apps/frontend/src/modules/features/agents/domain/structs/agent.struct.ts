import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';

export interface AgentStats {
  total: number;
  pending: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
  orphaned: number;
  lastRunAt: string;
  /** Last 30 days, oldest first; days without a finished job are absent. */
  daily: DailyOutcome[];
  /** `null`: no job ran yet (not a 0 ms run). */
  medianDurationMs: number | null;
  p95DurationMs: number | null;
}

export interface DailyOutcome {
  day: string;
  completed: number;
  failed: number;
  cancelled: number;
  orphaned: number;
  medianDurationMs: number | null;
}

/** The secret is returned once, at creation, and never again. */
export interface CreatedAgent {
  agent: AgentEntity;
  secret: string;
}

import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';

/**
 * Aggregate run stats for an agent, derived from the jobs it executed.
 */
export interface AgentStats {
  total: number;
  pending: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
  lastRunAt: string;
  /** Finished jobs per day (last 30 days, oldest first); gap days absent. */
  daily: DailyOutcome[];
}

/** One day of finished-job outcomes (day is an ISO date-time string). */
export interface DailyOutcome {
  day: string;
  completed: number;
  failed: number;
  cancelled: number;
}

/**
 * Result of creating an Agent. The secret is returned exactly once, at creation
 * time, and is never retrievable again.
 */
export interface CreatedAgent {
  agent: AgentEntity;
  secret: string;
}

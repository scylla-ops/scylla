/**
 * Domain model for a Agent (a specialized App that runs jobs).
 */
export interface Agent {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  /** Live agent stream presence — true while an agent is connected. */
  connected: boolean;
  /** Last activity timestamp; empty if the agent has never connected. */
  lastSeen: string;
  createdAt: string;
  updatedAt: string;
}

/**
 * Aggregate run stats for a agent, derived from the jobs it executed.
 */
export interface AgentStats {
  total: number;
  pending: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
  lastRunAt: string;
}

/**
 * Result of creating a Agent. The secret is returned exactly once, at creation
 * time, and is never retrievable again.
 */
export interface CreatedAgent {
  agent: Agent;
  secret: string;
}

/**
 * Domain entity for an Agent (a specialized App that runs jobs).
 */
export interface AgentEntity {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  /** Live agent stream presence — true while an agent is connected. */
  connected: boolean;
  /** Last activity timestamp; empty if the agent has never connected. */
  lastSeen: string;
  /** Jobs currently running on this agent; 0 while disconnected. */
  inFlight: number;
  createdAt: string;
  updatedAt: string;
}

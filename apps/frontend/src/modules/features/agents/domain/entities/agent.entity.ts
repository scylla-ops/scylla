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
  /**
   * Machine the agent last reported on connect. `null` until an agent that
   * speaks the hello has connected once; it outlives disconnects, so it
   * describes where the agent last ran.
   */
  host: AgentHost | null;
  createdAt: string;
  updatedAt: string;
}

/** The machine an agent runs on, as the agent described it. */
export interface AgentHost {
  /** Agent binary version. */
  version: string;
  os: string;
  arch: string;
  hostname: string;
  /** `null` when the agent could not read it — not "a machine with no CPU". */
  cpuCount: number | null;
  totalMemoryMb: number | null;
  reportedAt: string;
}

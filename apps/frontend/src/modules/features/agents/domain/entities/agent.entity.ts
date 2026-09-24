export interface AgentEntity {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  connected: boolean;
  /** Empty if it never connected. */
  lastSeen: string;
  /** 0 while disconnected. */
  inFlight: number;
  /** `null` until the agent said hello. */
  host: AgentHost | null;
  createdAt: string;
  updatedAt: string;
}

export interface AgentHost {
  version: string;
  os: string;
  arch: string;
  hostname: string;
  /** `null` when the agent could not read it. */
  cpuCount: number | null;
  totalMemoryMb: number | null;
  reportedAt: string;
}

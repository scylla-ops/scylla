/**
 * Domain model for a Worker (Agent)
 */
export interface Worker {
  agentId: string;
  hostname: string;
  status: string;
  lastSeenAt: string;
  createdAt: string;
  updatedAt: string;
}

export interface WorkersListResponse {
  workers: Worker[];
  pagination?: {
    total?: number;
    hasMore?: boolean;
  };
}

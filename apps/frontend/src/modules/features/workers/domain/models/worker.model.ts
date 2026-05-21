/**
 * Domain model for a Worker (a specialized App that runs jobs).
 */
export interface Worker {
  id: string;
  organizationId: string;
  name: string;
  isActive: boolean;
  /** Live worker stream presence — true while an agent is connected. */
  connected: boolean;
  /** Last activity timestamp; empty if the worker has never connected. */
  lastSeen: string;
  createdAt: string;
  updatedAt: string;
}

/**
 * Aggregate run stats for a worker, derived from the jobs it executed.
 */
export interface WorkerStats {
  total: number;
  pending: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
  lastRunAt: string;
}

/**
 * Result of creating a Worker. The secret is returned exactly once, at creation
 * time, and is never retrievable again.
 */
export interface CreatedWorker {
  worker: Worker;
  secret: string;
}

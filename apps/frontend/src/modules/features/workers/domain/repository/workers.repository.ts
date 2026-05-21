import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedWorker,
  Worker,
  WorkerStats,
} from '@/modules/features/workers/domain/models/worker.model.ts';

/**
 * Repository interface for Workers. Workers are organization-scoped.
 */
export interface WorkersRepository {
  listWorkers(organizationId: string): Promise<ScyllaResult<Worker[]>>;
  getWorker(workerId: string): Promise<ScyllaResult<Worker>>;
  getWorkerStats(workerId: string): Promise<ScyllaResult<WorkerStats>>;
  createWorker(organizationId: string, name: string): Promise<ScyllaResult<CreatedWorker>>;
  deleteWorker(workerId: string): Promise<ScyllaResult<boolean>>;
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  Worker,
  WorkersListResponse,
} from '@/modules/features/workers/domain/models/worker.model.ts';

/**
 * Repository interface for Workers
 * Abstraction layer for all worker data operations
 */
export interface WorkersRepository {
  listWorkers(pagination?: {
    page?: number;
    pageSize?: number;
  }): Promise<ScyllaResult<WorkersListResponse>>;
  getWorker(workerId: string): Promise<ScyllaResult<Worker>>;
}

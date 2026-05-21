import type {
  WorkerStats as ProtoWorkerStats,
  WorkerView as ProtoWorkerView,
} from '@/generated/worker_admin.ts';
import type { Worker, WorkerStats } from '@/modules/features/workers/domain/models/worker.model.ts';

/** Maps gRPC WorkerAdmin messages to the domain Worker models. */
export class GrpcWorkerMapper {
  static toDomain(w: ProtoWorkerView): Worker {
    return {
      id: w.id,
      organizationId: w.organizationId,
      name: w.name,
      isActive: w.isActive,
      connected: w.connected,
      lastSeen: w.lastSeen,
      createdAt: w.createdAt,
      updatedAt: w.updatedAt,
    };
  }

  static statsToDomain(s: ProtoWorkerStats): WorkerStats {
    return {
      total: Number(s.total),
      pending: Number(s.pending),
      running: Number(s.running),
      completed: Number(s.completed),
      failed: Number(s.failed),
      cancelled: Number(s.cancelled),
      lastRunAt: s.lastRunAt,
    };
  }
}

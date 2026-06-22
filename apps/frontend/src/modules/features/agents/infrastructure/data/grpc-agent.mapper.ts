import type {
  AgentStats as ProtoAgentStats,
  AgentView as ProtoAgentView,
} from '@/generated/agent_admin.ts';
import type { Agent, AgentStats } from '@/modules/features/agents/domain/models/agent.model.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

/** Maps gRPC AgentAdmin messages to the domain Agent models. */
export class GrpcAgentMapper {
  static toDomain(w: ProtoAgentView): Agent {
    return {
      id: idValue(w.id),
      organizationId: idValue(w.organizationId),
      name: w.name,
      isActive: w.isActive,
      connected: w.connected,
      lastSeen: timestampToIso(w.lastSeen),
      createdAt: timestampToIso(w.createdAt),
      updatedAt: timestampToIso(w.updatedAt),
    };
  }

  static statsToDomain(s: ProtoAgentStats): AgentStats {
    return {
      total: Number(s.total),
      pending: Number(s.pending),
      running: Number(s.running),
      completed: Number(s.completed),
      failed: Number(s.failed),
      cancelled: Number(s.cancelled),
      lastRunAt: timestampToIso(s.lastRunAt),
      daily: s.daily.map(d => ({
        day: timestampToIso(d.day),
        completed: Number(d.completed),
        failed: Number(d.failed),
        cancelled: Number(d.cancelled),
      })),
    };
  }
}

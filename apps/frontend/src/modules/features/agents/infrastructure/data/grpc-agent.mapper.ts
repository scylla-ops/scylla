import type {
  AgentStats as ProtoAgentStats,
  AgentView as ProtoAgentView,
} from '@/generated/agent_admin.ts';
import type { Agent, AgentStats } from '@/modules/features/agents/domain/models/agent.model.ts';

/** Maps gRPC AgentAdmin messages to the domain Agent models. */
export class GrpcAgentMapper {
  static toDomain(w: ProtoAgentView): Agent {
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

  static statsToDomain(s: ProtoAgentStats): AgentStats {
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

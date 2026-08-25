import type {
  AgentStats as ProtoAgentStats,
  Agent as ProtoAgent,
  AgentHost as ProtoAgentHost,
} from '@/generated/scylla/agent/v1/agent_admin.ts';
import type {
  AgentEntity,
  AgentHost,
} from '@/modules/features/agents/domain/entities/agent.entity.ts';
import type { AgentStats } from '@/modules/features/agents/domain/structs/agent.struct.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

/** Maps gRPC AgentAdmin messages to the domain Agent models. */
export class GrpcAgentMapper {
  static toDomain(w: ProtoAgent): AgentEntity {
    return {
      id: idValue(w.agentId),
      organizationId: idValue(w.organizationId),
      name: w.name,
      isActive: w.isActive,
      connected: w.connected,
      lastSeen: timestampToIso(w.lastSeen),
      inFlight: w.inFlight,
      host: w.host ? GrpcAgentMapper.hostToDomain(w.host) : null,
      createdAt: timestampToIso(w.createdAt),
      updatedAt: timestampToIso(w.updatedAt),
    };
  }

  private static hostToDomain(h: ProtoAgentHost): AgentHost {
    return {
      version: h.version,
      os: h.os,
      arch: h.arch,
      hostname: h.hostname,
      cpuCount: h.cpuCount > 0 ? h.cpuCount : null,
      totalMemoryMb: h.totalMemoryMb > 0n ? Number(h.totalMemoryMb) : null,
      reportedAt: timestampToIso(h.reportedAt),
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
      orphaned: Number(s.orphaned),
      lastRunAt: timestampToIso(s.lastRunAt),
      medianDurationMs: optionalCount(s.medianDurationMs),
      p95DurationMs: optionalCount(s.p95DurationMs),
      daily: s.daily.map(d => ({
        day: timestampToIso(d.day),
        completed: Number(d.completed),
        failed: Number(d.failed),
        cancelled: Number(d.cancelled),
        orphaned: Number(d.orphaned),
        medianDurationMs: optionalCount(d.medianDurationMs),
      })),
    };
  }
}

/** Absent on the wire means unknown, which must stay `null` and not become 0. */
const optionalCount = (v: bigint | undefined): number | null =>
  v === undefined ? null : Number(v);

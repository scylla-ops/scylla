import type { AgentResponse, ListAgentsResponse } from '@/generated/agent.ts';
import type {
  Worker,
  WorkersListResponse,
} from '@/modules/features/workers/domain/models/worker.model.ts';

/**
 * Mapper for converting gRPC agent responses to domain Worker models
 */
export class GrpcWorkerMapper {
  /**
   * Map single agent gRPC response to Worker domain model
   */
  static toDomain(agent: AgentResponse): Worker {
    return {
      agentId: agent.agentId,
      hostname: agent.hostname,
      status: agent.status,
      lastSeenAt: agent.lastSeenAt,
      createdAt: agent.createdAt,
      updatedAt: agent.updatedAt,
    };
  }

  /**
   * Map list agents gRPC response to WorkersListResponse domain model
   */
  static toDomainList(response: ListAgentsResponse): WorkersListResponse {
    return {
      workers: response.agents?.map(agent => this.toDomain(agent)) || [],
      pagination: response.pagination
        ? {
            total: response.pagination.totalCount,
            hasMore: response.pagination.hasNext,
          }
        : undefined,
    };
  }
}

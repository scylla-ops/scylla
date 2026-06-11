import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';
import type { AgentsRemoteDataSource } from '@/modules/features/agents/infrastructure/data/agents-remote.data-source.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedAgent,
  Agent,
  AgentStats,
} from '@/modules/features/agents/domain/models/agent.model.ts';

/** AgentsRepository backed by the remote (gRPC) data source. */
export class DefaultAgentsRepository implements AgentsRepository {
  constructor(private remoteDataSource: AgentsRemoteDataSource) {}

  listAgents(organizationId: string): Promise<ScyllaResult<Agent[]>> {
    return this.remoteDataSource.listAgents(organizationId);
  }

  getAgent(agentId: string): Promise<ScyllaResult<Agent>> {
    return this.remoteDataSource.getAgent(agentId);
  }

  getAgentStats(agentId: string): Promise<ScyllaResult<AgentStats>> {
    return this.remoteDataSource.getAgentStats(agentId);
  }

  createAgent(organizationId: string, name: string): Promise<ScyllaResult<CreatedAgent>> {
    return this.remoteDataSource.createAgent(organizationId, name);
  }

  deleteAgent(agentId: string): Promise<ScyllaResult<boolean>> {
    return this.remoteDataSource.deleteAgent(agentId);
  }
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedAgent,
  AgentStats,
} from '@/modules/features/agents/domain/structs/agent.struct.ts';
import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';

/**
 * Repository interface for Agents. Agents are organization-scoped.
 */
export interface AgentsRepository {
  listAgents(organizationId: string): Promise<ScyllaResult<AgentEntity[]>>;
  getAgent(agentId: string): Promise<ScyllaResult<AgentEntity>>;
  getAgentStats(agentId: string): Promise<ScyllaResult<AgentStats>>;
  createAgent(organizationId: string, name: string): Promise<ScyllaResult<CreatedAgent>>;
  deleteAgent(agentId: string): Promise<ScyllaResult<void>>;
}

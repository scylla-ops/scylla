import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedAgent,
  Agent,
  AgentStats,
} from '@/modules/features/agents/domain/models/agent.model.ts';

/**
 * Repository interface for Agents. Agents are organization-scoped.
 */
export interface AgentsRepository {
  listAgents(organizationId: string): Promise<ScyllaResult<Agent[]>>;
  getAgent(agentId: string): Promise<ScyllaResult<Agent>>;
  getAgentStats(agentId: string): Promise<ScyllaResult<AgentStats>>;
  createAgent(organizationId: string, name: string): Promise<ScyllaResult<CreatedAgent>>;
  deleteAgent(agentId: string): Promise<ScyllaResult<boolean>>;
}

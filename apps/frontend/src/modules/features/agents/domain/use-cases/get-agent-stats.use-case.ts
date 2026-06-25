import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentStats } from '@/modules/features/agents/domain/structs/agent.struct.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: fetch the run stats of an agent. */
export class GetAgentStatsUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(agentId: string): Promise<ScyllaResult<AgentStats>> {
    return this.repository.getAgentStats(agentId);
  }
}

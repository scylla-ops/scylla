import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Agent } from '@/modules/features/agents/domain/models/agent.model.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: fetch a single agent by id. */
export class GetAgentUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(agentId: string): Promise<ScyllaResult<Agent>> {
    return this.repository.getAgent(agentId);
  }
}

import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: fetch a single agent by id. */
export class GetAgentUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(agentId: string): Promise<ScyllaResult<AgentEntity>> {
    return this.repository.getAgent(agentId);
  }
}

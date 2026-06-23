import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: list the agents of an organization. */
export class GetAgentsUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(organizationId: string): Promise<ScyllaResult<AgentEntity[]>> {
    return this.repository.listAgents(organizationId);
  }
}

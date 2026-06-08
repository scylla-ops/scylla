import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Agent } from '@/modules/features/agents/domain/models/agent.model.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: list the agents of an organization. */
export class GetAgentsUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(organizationId: string): Promise<ScyllaResult<Agent[]>> {
    return this.repository.listAgents(organizationId);
  }
}

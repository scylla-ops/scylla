import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CreatedAgent } from '@/modules/features/agents/domain/models/agent.model.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: create an agent. Returns the one-time secret. */
export class CreateAgentUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(organizationId: string, name: string): Promise<ScyllaResult<CreatedAgent>> {
    return this.repository.createAgent(organizationId, name);
  }
}

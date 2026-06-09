import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';

/** Use case: delete an agent (revokes its grants, disconnects its agent). */
export class DeleteAgentUseCase {
  constructor(private readonly repository: AgentsRepository) {}

  execute(agentId: string): Promise<ScyllaResult<boolean>> {
    return this.repository.deleteAgent(agentId);
  }
}

import { AgentsRemoteDataSourceImpl } from '@/modules/features/agents/infrastructure/data/agents-remote.data-source.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { DefaultAgentsRepository } from '@/modules/features/agents/infrastructure/repository/default-agents.repository.ts';
import { GetAgentsUseCase } from '@/modules/features/agents/domain/use-cases/get-agents.use-case.ts';
import { GetAgentUseCase } from '@/modules/features/agents/domain/use-cases/get-agent.use-case.ts';
import { GetAgentStatsUseCase } from '@/modules/features/agents/domain/use-cases/get-agent-stats.use-case.ts';
import { CreateAgentUseCase } from '@/modules/features/agents/domain/use-cases/create-agent.use-case.ts';
import { DeleteAgentUseCase } from '@/modules/features/agents/domain/use-cases/delete-agent.use-case.ts';

const dataSource = new AgentsRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new DefaultAgentsRepository(dataSource);

export const AgentsModule = {
  domain: {
    getAgents: new GetAgentsUseCase(repository),
    getAgent: new GetAgentUseCase(repository),
    getAgentStats: new GetAgentStatsUseCase(repository),
    createAgent: new CreateAgentUseCase(repository),
    deleteAgent: new DeleteAgentUseCase(repository),
  },
};

import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { AgentAdminServiceClient } from '@/generated/agent_admin.client.ts';
import type {
  CreatedAgent,
  AgentStats,
} from '@/modules/features/agents/domain/structs/agent.struct.ts';
import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';
import { GrpcAgentMapper } from './grpc-agent.mapper.ts';

/** Remote data source for agents over gRPC. Shares the repository contract. */
export type AgentsRemoteDataSource = AgentsRepository;

export class AgentsRemoteDataSourceImpl implements AgentsRemoteDataSource {
  constructor(private grpcTransport: CoreGrpcTransport) {}

  listAgents(organizationId: string): Promise<ScyllaResult<AgentEntity[]>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.listAgents({ organizationId: wrapId(organizationId) }).response;
      return response.agents.map(agent => GrpcAgentMapper.toDomain(agent));
    }, 'Failed to list agents');
  }

  getAgent(agentId: string): Promise<ScyllaResult<AgentEntity>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.getAgent({ id: wrapId(agentId) }).response;
      return GrpcAgentMapper.toDomain(response);
    }, 'Failed to fetch agent');
  }

  getAgentStats(agentId: string): Promise<ScyllaResult<AgentStats>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.getAgentStats({ id: wrapId(agentId) }).response;
      return GrpcAgentMapper.statsToDomain(response);
    }, 'Failed to fetch agent stats');
  }

  createAgent(organizationId: string, name: string): Promise<ScyllaResult<CreatedAgent>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.createAgent({ organizationId: wrapId(organizationId), name })
        .response;
      if (!response.agent) throw new Error('CreateAgent returned no agent');
      return { agent: GrpcAgentMapper.toDomain(response.agent), secret: response.secret };
    }, 'Failed to create agent');
  }

  deleteAgent(agentId: string): Promise<ScyllaResult<boolean>> {
    return ScyllaResult.tryAsync(async () => {
      const client = new AgentAdminServiceClient(this.grpcTransport.getTransport());
      const response = await client.deleteAgent({ id: wrapId(agentId) }).response;
      return response.deleted;
    }, 'Failed to delete agent');
  }
}

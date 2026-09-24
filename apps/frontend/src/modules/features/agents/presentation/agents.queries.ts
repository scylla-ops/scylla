import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { Permission, can } from '@platform/authz';
import type { AgentEntity } from '../domain/entities/agent.entity.ts';
import type { AgentStats, CreatedAgent } from '../domain/structs/agent.struct.ts';
import type { AgentsModule } from '../agents.module.ts';

// Resolved per call: tests swap the registry.
const repository = () => getModuleDomain<typeof AgentsModule.domain>('agents').agentsRepository;

export const AGENTS_QUERY_KEY = (organizationId: string) => ['agents', organizationId] as const;
export const AGENT_QUERY_KEY = (agentId: string) => ['agents', 'detail', agentId] as const;
export const AGENT_STATS_QUERY_KEY = (agentId: string) => ['agents', 'stats', agentId] as const;

const LIVE = { refetchInterval: 10_000, refetchIntervalInBackground: false } as const;

export const agentQueries = {
  /**
   * Not asked without `LIST_AGENTS` (a sure denial, toasted on every page). So an
   * empty list means "no agents", never "not allowed": branch on the permission first.
   */
  byOrganization: (organizationId: string) =>
    queryOptions<AgentEntity[]>({
      queryKey: AGENTS_QUERY_KEY(organizationId),
      enabled: !!organizationId && can(Permission.LIST_AGENTS),
      ...LIVE,
      queryFn: async () => (await repository().listAgents(organizationId)).unwrap(),
    }),

  byId: (agentId: string) =>
    queryOptions<AgentEntity>({
      queryKey: AGENT_QUERY_KEY(agentId),
      enabled: !!agentId,
      ...LIVE,
      queryFn: async () => (await repository().getAgent(agentId)).unwrap(),
    }),

  statsOf: (agentId: string) =>
    queryOptions<AgentStats>({
      queryKey: AGENT_STATS_QUERY_KEY(agentId),
      enabled: !!agentId && can(Permission.READ_APP_STATS),
      ...LIVE,
      queryFn: async () => (await repository().getAgentStats(agentId)).unwrap(),
    }),
};

const invalidateList = (organizationId: string) =>
  getQueryClient().invalidateQueries({ queryKey: AGENTS_QUERY_KEY(organizationId) });

export const agentMutations = {
  /** The secret passes through once and is never stored. */
  create: (organizationId: string) =>
    mutationOptions({
      mutationFn: async (name: string): Promise<CreatedAgent> =>
        (await repository().createAgent(organizationId, name)).unwrap(),
      onSuccess: () => void invalidateList(organizationId),
    }),

  remove: (organizationId: string) =>
    mutationOptions({
      mutationFn: async (agentId: string) => (await repository().deleteAgent(agentId)).unwrap(),
      onSuccess: () => void invalidateList(organizationId),
    }),
};

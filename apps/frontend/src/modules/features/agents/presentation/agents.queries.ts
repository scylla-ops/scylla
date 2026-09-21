import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { Permission, can } from '@platform/authz';
import type { AgentEntity } from '../domain/entities/agent.entity.ts';
import type { AgentStats, CreatedAgent } from '../domain/structs/agent.struct.ts';
import type { AgentsModule } from '../agents.module.ts';

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what `use-agents.ts` was, with the framework taken out — and the
 * reason it had to go first: `pipeline` and `dashboard` are still React and
 * consume these through the barrel. A `queryOptions` object has no framework in
 * it, so `useQuery` takes it unchanged and both halves share one cache entry.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const repository = () => getModuleDomain<typeof AgentsModule.domain>('agents').agentsRepository;

export const AGENTS_QUERY_KEY = (organizationId: string) => ['agents', organizationId] as const;
export const AGENT_QUERY_KEY = (agentId: string) => ['agents', 'detail', agentId] as const;
export const AGENT_STATS_QUERY_KEY = (agentId: string) => ['agents', 'stats', agentId] as const;

/** Keep online/offline + last-seen fresh; pause when the tab is hidden. */
const LIVE = { refetchInterval: 10_000, refetchIntervalInBackground: false } as const;

export const agentQueries = {
  /**
   * An organization's agents.
   *
   * **The permission check is part of the query, not of a caller.** `ListAgents`
   * is enforced server-side, so asking without `LIST_AGENTS` is a guaranteed
   * PERMISSION_DENIED — and the global query error handler would toast it on
   * every page that merely *peeks* at agents. Not asking also keeps an empty
   * list meaning "no agents", never "not allowed to look": a caller reporting
   * on connectivity must branch on the permission first (see `NoAgentsBanner`).
   *
   * A React consumer must subscribe to the permissions store itself —
   * `useCan(Permission.LIST_AGENTS)` — or it will never re-render when the
   * permissions land and the query will stay disabled for good. In Svelte
   * `can()` is reactive and a `$derived` recomputes on its own.
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

  /** Aggregate run stats. Gated the same way, on `READ_APP_STATS`. */
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
  /** The plaintext passes through here once, on its way out. Never stored. */
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

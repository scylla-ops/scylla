import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

const WORKERS_QUERY_KEY = 'agents';

export function useAgents() {
  const { agents } = useDependencies();
  const organizationId = useContextStore(state => state.organization.id);
  const queryClient = useQueryClient();
  // `ListAgents` is enforced server-side, so asking without LIST_AGENTS is a
  // guaranteed PERMISSION_DENIED — and the global query error handler would
  // toast it on every page that only *peeks* at agents. Not asking also keeps
  // an empty list meaning "no agents", never "not allowed to look": callers
  // that report on connectivity must branch on `canListAgents` first.
  const { can, ready } = useAuthorization();
  const canListAgents = ready && can(Permission.LIST_AGENTS);

  const query = useQuery({
    queryKey: [WORKERS_QUERY_KEY, organizationId],
    enabled: !!organizationId && canListAgents,
    // Keep online/offline + last-seen fresh; pause when the tab is hidden.
    refetchInterval: 10_000,
    refetchIntervalInBackground: false,
    queryFn: async () => {
      const result = await agents.getAgents.execute(organizationId ?? '');
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });

  const createAgent = useMutation({
    mutationFn: async (name: string) => {
      const result = await agents.createAgent.execute(organizationId ?? '', name);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: [WORKERS_QUERY_KEY, organizationId] }),
  });

  const deleteAgent = useMutation({
    mutationFn: async (agentId: string) => {
      const result = await agents.deleteAgent.execute(agentId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: [WORKERS_QUERY_KEY, organizationId] }),
  });

  return {
    agents: query.data ?? [],
    /** False while permissions are still unknown, and for callers who may not look. */
    canListAgents,
    isLoading: !ready || query.isLoading,
    isError: query.isError,
    error: query.error,
    createAgent,
    deleteAgent,
  };
}

export function useAgent(agentId: string) {
  const { agents } = useDependencies();

  return useQuery({
    queryKey: [WORKERS_QUERY_KEY, 'detail', agentId],
    enabled: !!agentId,
    refetchInterval: 10_000,
    refetchIntervalInBackground: false,
    queryFn: async () => {
      const result = await agents.getAgent.execute(agentId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });
}

export function useAgentStats(agentId: string) {
  const { agents } = useDependencies();

  return useQuery({
    queryKey: [WORKERS_QUERY_KEY, 'stats', agentId],
    enabled: !!agentId,
    refetchInterval: 10_000,
    refetchIntervalInBackground: false,
    queryFn: async () => {
      const result = await agents.getAgentStats.execute(agentId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });
}

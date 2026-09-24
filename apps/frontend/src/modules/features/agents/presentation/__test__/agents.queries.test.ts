// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { runMutationFn, runOnSuccess, runQueryFn } from '@/test/queries.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentsRepository } from '../../domain/repository/agents.repository.ts';
import {
  AGENTS_QUERY_KEY,
  AGENT_QUERY_KEY,
  AGENT_STATS_QUERY_KEY,
  agentMutations,
  agentQueries,
} from '../agents.queries.ts';

const agent = {
  id: 'agent-1',
  organizationId: 'org-1',
  name: 'runner-1',
  isActive: true,
  connected: true,
  lastSeen: '',
  inFlight: 0,
  host: null,
  createdAt: '',
  updatedAt: '',
};

let repository: AgentsRepository;
let invalidate: ReturnType<typeof vi.fn>;

/** Drives the real permissions store — never a mocked `can`. */
const grantEverything = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

beforeEach(() => {
  repository = {
    listAgents: vi.fn().mockResolvedValue(ScyllaResult.success([agent])),
    getAgent: vi.fn().mockResolvedValue(ScyllaResult.success(agent)),
    getAgentStats: vi.fn().mockResolvedValue(ScyllaResult.success({ daily: [] })),
    createAgent: vi.fn().mockResolvedValue(ScyllaResult.success({ agent, secret: 'sk-once' })),
    deleteAgent: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
  };

  setDependencyRegistry({ agents: { agentsRepository: repository } });

  const queryClient = new QueryClient();
  invalidate = vi.fn();
  queryClient.invalidateQueries = invalidate as unknown as QueryClient['invalidateQueries'];
  setQueryClient(queryClient);

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  grantEverything();
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
  permissionsStore.setState({ permissions: null });
});

describe('agentQueries', () => {
  it('lists the agents of one organization', async () => {
    const options = agentQueries.byOrganization('org-1');

    expect(options.queryKey).toEqual(AGENTS_QUERY_KEY('org-1'));
    await expect(runQueryFn(options)).resolves.toEqual([agent]);
    expect(repository.listAgents).toHaveBeenCalledWith('org-1');
  });

  it('refuses to ask for a list the caller may not see', () => {
    // `ListAgents` is enforced server-side, so asking without LIST_AGENTS is a
    // guaranteed PERMISSION_DENIED — and the global error handler would toast
    // it on every page that merely peeks at agents.
    permissionsStore.setState({ permissions: { scopes: [] } });

    expect(agentQueries.byOrganization('org-1').enabled).toBe(false);
  });

  it('gates the stats query on its own permission, separately from the list', () => {
    permissionsStore.setState({
      permissions: {
        scopes: [
          {
            scope: PermissionScope.SYSTEM,
            scopeId: '',
            access: { kind: 'restricted', permissions: [Permission.LIST_AGENTS] },
          },
        ],
      },
    });

    expect(agentQueries.byOrganization('org-1').enabled).toBe(true);
    expect(agentQueries.statsOf('agent-1').enabled).toBe(false);
  });

  it('stays disabled without an id, rather than fetching for an empty one', () => {
    expect(agentQueries.byOrganization('').enabled).toBe(false);
    expect(agentQueries.byId('').enabled).toBe(false);
    expect(agentQueries.statsOf('').enabled).toBe(false);
  });

  it('keeps polling so online/offline and last-seen stay fresh, but not in a hidden tab', () => {
    const options = agentQueries.byOrganization('org-1');

    expect(options.refetchInterval).toBe(10_000);
    expect(options.refetchIntervalInBackground).toBe(false);
  });

  it('gives an agent, its detail and its stats three distinct cache entries', () => {
    expect(AGENT_QUERY_KEY('agent-1')).not.toEqual(AGENT_STATS_QUERY_KEY('agent-1'));
    expect(AGENTS_QUERY_KEY('org-1')).not.toEqual(AGENT_QUERY_KEY('agent-1'));
  });
});

describe('agentMutations', () => {
  it('creates an agent and invalidates the list it now belongs to', async () => {
    const options = agentMutations.create('org-1');

    const created = await runMutationFn(options, 'runner-1');
    expect(repository.createAgent).toHaveBeenCalledWith('org-1', 'runner-1');
    expect(created.secret).toBe('sk-once');

    runOnSuccess(options, created, 'runner-1');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: AGENTS_QUERY_KEY('org-1') });
  });

  it('deletes an agent and refreshes the list', async () => {
    const options = agentMutations.remove('org-1');

    await runMutationFn(options, 'agent-1');
    expect(repository.deleteAgent).toHaveBeenCalledWith('agent-1');

    runOnSuccess(options, undefined, 'agent-1');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: AGENTS_QUERY_KEY('org-1') });
  });
});

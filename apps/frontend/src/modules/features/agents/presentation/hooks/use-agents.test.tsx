import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { useContextStore } from '@platform/context';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useAgents, useAgent, useAgentStats } from './use-agents';
import type { AgentsRepository } from '@/modules/features/agents/domain/repository/agents.repository.ts';
import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';

const ORG_ID = 'org-1';

const agent = (overrides: Partial<AgentEntity> = {}): AgentEntity => ({
  id: 'agent-1',
  organizationId: ORG_ID,
  name: 'agent-1',
  isActive: true,
  connected: true,
  lastSeen: '2026-01-01T00:00:00.000Z',
  inFlight: 0,
  host: null,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

/**
 * Each mock is its own named const, and assertions reference those consts
 * directly rather than `repository.method` (which trips
 * @typescript-eslint/unbound-method — the interface declares methods, not
 * arrow-typed properties, so a bare property access reads as an unbound
 * method reference to the linter).
 */
const makeFakeRepository = (overrides: Partial<AgentsRepository> = {}) => {
  const listAgents = vi.fn().mockResolvedValue(ScyllaResult.success([agent()]));
  const getAgent = vi.fn().mockResolvedValue(ScyllaResult.success(agent()));
  const getAgentStats = vi.fn().mockResolvedValue(
    ScyllaResult.success({
      total: 1,
      pending: 0,
      running: 0,
      completed: 1,
      failed: 0,
      cancelled: 0,
      orphaned: 0,
      lastRunAt: '2026-01-01T00:00:00.000Z',
      daily: [],
      medianDurationMs: 100,
      p95DurationMs: 200,
    }),
  );
  const createAgent = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ agent: agent(), secret: 's3cr3t' }));
  const deleteAgent = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  const repository: AgentsRepository = {
    listAgents,
    getAgent,
    getAgentStats,
    createAgent,
    deleteAgent,
    ...overrides,
  };

  return { repository, listAgents, getAgent, getAgentStats, createAgent, deleteAgent };
};

/** Grants every permission at the SYSTEM scope, or none when `canDoAnything` is false. */
const setPermissions = (canDoAnything: boolean) => {
  usePermissionsStore.setState({
    permissions: canDoAnything
      ? { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] }
      : { scopes: [] },
  });
};

const wrapperFor = (repository: AgentsRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider registry={{ agents: { agentsRepository: repository } }}>
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return { Wrapper, queryClient };
};

beforeEach(() => {
  useContextStore.setState({ organization: { id: ORG_ID, name: 'Org' } });
  setPermissions(true);
});

describe('useAgents', () => {
  it('does not call listAgents while permissions are still unknown', () => {
    usePermissionsStore.setState({ permissions: null });
    const { repository, listAgents } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    expect(listAgents).not.toHaveBeenCalled();
    expect(result.current.canListAgents).toBe(false);
  });

  it('does not call listAgents when the user lacks LIST_AGENTS', () => {
    setPermissions(false);
    const { repository, listAgents } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    expect(listAgents).not.toHaveBeenCalled();
    expect(result.current.canListAgents).toBe(false);
    expect(result.current.agents).toEqual([]);
  });

  it('lists the current organization\'s agents once permission + org are known', async () => {
    const { repository, listAgents } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.agents).toHaveLength(1));

    expect(listAgents).toHaveBeenCalledWith(ORG_ID);
    expect(result.current.agents[0].id).toBe('agent-1');
    expect(result.current.canListAgents).toBe(true);
  });

  it('surfaces a repository error via isError/error rather than throwing to the caller', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      listAgents: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
  });

  it('createAgent calls the repository and invalidates the agents query on success', async () => {
    const { repository, createAgent } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.agents).toHaveLength(1));
    await result.current.createAgent.mutateAsync('new-agent');

    expect(createAgent).toHaveBeenCalledWith(ORG_ID, 'new-agent');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['agents', ORG_ID] });
  });

  it('deleteAgent calls the repository and invalidates the agents query on success', async () => {
    const { repository, deleteAgent } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.agents).toHaveLength(1));
    await result.current.deleteAgent.mutateAsync('agent-1');

    expect(deleteAgent).toHaveBeenCalledWith('agent-1');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['agents', ORG_ID] });
  });

  it('a failed createAgent rejects the mutation instead of silently succeeding', async () => {
    const error = new ScyllaError('name taken', { cause: { code: 'ALREADY_EXISTS' } });
    const { repository } = makeFakeRepository({
      createAgent: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.agents).toHaveLength(1));
    await expect(result.current.createAgent.mutateAsync('dup')).rejects.toBe(error);
  });
});

describe('useAgent', () => {
  it('fetches a single agent by id', async () => {
    const { repository, getAgent } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgent('agent-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.data?.id).toBe('agent-1'));
    expect(getAgent).toHaveBeenCalledWith('agent-1');
  });

  it('does not fetch when agentId is empty', () => {
    const { repository, getAgent } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useAgent(''), { wrapper: Wrapper });

    expect(getAgent).not.toHaveBeenCalled();
  });
});

describe('useAgentStats', () => {
  it('fetches stats for the given agent', async () => {
    const { repository, getAgentStats } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAgentStats('agent-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.data?.total).toBe(1));
    expect(getAgentStats).toHaveBeenCalledWith('agent-1');
  });
});

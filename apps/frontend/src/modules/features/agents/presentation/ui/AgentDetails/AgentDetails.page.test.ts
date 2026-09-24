import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentsRepository } from '../../../domain/repository/agents.repository.ts';
import AgentDetailsPage from './AgentDetails.page.svelte';

const agent = (overrides: Record<string, unknown> = {}) => ({
  id: 'agent-1',
  organizationId: 'org-1',
  name: 'runner-1',
  isActive: true,
  connected: true,
  lastSeen: '2026-01-01T00:00:00.000Z',
  inFlight: 0,
  host: null,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-02T00:00:00.000Z',
  ...overrides,
});

const stats = (overrides: Record<string, unknown> = {}) => ({
  daily: [],
  completed: 4,
  failed: 1,
  cancelled: 0,
  lastRunAt: '2026-01-02T00:00:00.000Z',
  ...overrides,
});

let getAgent: ReturnType<typeof vi.fn>;
let getAgentStats: ReturnType<typeof vi.fn>;
let deleteAgent: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;
let nav: ReturnType<typeof installTestNavigator>;

const grant = (permissions: Permission[] | 'all') =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access:
            permissions === 'all' ? { kind: 'fullControl' } : { kind: 'restricted', permissions },
        },
      ],
    },
  });

beforeEach(() => {
  getAgent = vi.fn().mockResolvedValue(ScyllaResult.success(agent()));
  getAgentStats = vi.fn().mockResolvedValue(ScyllaResult.success(stats()));
  deleteAgent = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    agents: {
      agentsRepository: {
        getAgent,
        getAgentStats,
        deleteAgent,
        listAgents: vi.fn(),
        createAgent: vi.fn(),
      } as unknown as AgentsRepository,
    },
  });

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  nav = installTestNavigator({ pathname: '/acme/agents/agent-1' });
  grant('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  nav.restore();
  permissionsStore.setState({ permissions: null });
});

describe('AgentDetailsPage', () => {
  it('reads the agent named by the route parameter', async () => {
    render(AgentDetailsPage, { agentId: 'agent-1' });

    expect(await screen.findByRole('heading', { name: 'runner-1' })).toBeInTheDocument();
    expect(getAgent).toHaveBeenCalledWith('agent-1');
  });

  it('says an agent is online, with when it was last seen', async () => {
    render(AgentDetailsPage, { agentId: 'agent-1' });

    expect(await screen.findByText(/online/)).toBeInTheDocument();
    expect(screen.getByText(/seen/)).toBeInTheDocument();
  });

  it('says an agent is offline and down since when', async () => {
    getAgent.mockResolvedValue(ScyllaResult.success(agent({ connected: false })));
    render(AgentDetailsPage, { agentId: 'agent-1' });

    expect(await screen.findByText(/offline/)).toBeInTheDocument();
    expect(screen.getByText(/down/)).toBeInTheDocument();
  });

  it('shows the job stats section to a caller who may read them', async () => {
    render(AgentDetailsPage, { agentId: 'agent-1' });

    expect(await screen.findByRole('heading', { name: 'Job stats' })).toBeInTheDocument();
  });

  it('hides the stats section entirely without READ_APP_STATS, rather than showing it empty', async () => {
    grant([Permission.READ_APP]);
    render(AgentDetailsPage, { agentId: 'agent-1' });

    await screen.findByRole('heading', { name: 'runner-1' });
    expect(screen.queryByRole('heading', { name: 'Job stats' })).not.toBeInTheDocument();
  });

  it('always explains how to run a worker for this agent', async () => {
    render(AgentDetailsPage, { agentId: 'agent-1' });

    expect(await screen.findByRole('heading', { name: 'Run this agent' })).toBeInTheDocument();
  });

  it('reports a failed load rather than a blank page', async () => {
    getAgent.mockResolvedValue(
      ScyllaResult.error(new ScyllaError('boom', { cause: { code: 'INTERNAL' } })),
    );
    render(AgentDetailsPage, { agentId: 'agent-1' });

    expect(await screen.findByText('Error loading agent')).toBeInTheDocument();
  });

  it('leaves for the list when the agent no longer exists', async () => {
    getAgent.mockResolvedValue(
      ScyllaResult.error(new ScyllaError('gone', { cause: { code: 'NOT_FOUND' } })),
    );
    render(AgentDetailsPage, { agentId: 'agent-1' });

    // A deleted agent redirects to the list.
    await vi.waitFor(() => expect(nav.navigate).toHaveBeenCalledWith('..', expect.anything()));
  });

  it('hides the delete button without DELETE_APP', async () => {
    grant([Permission.READ_APP, Permission.READ_APP_STATS]);
    render(AgentDetailsPage, { agentId: 'agent-1' });

    await screen.findByRole('heading', { name: 'runner-1' });
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
  });

  it('confirms before deleting, then returns to the list', async () => {
    render(AgentDetailsPage, { agentId: 'agent-1' });
    await screen.findByRole('heading', { name: 'runner-1' });

    await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('Delete agent?')).toBeInTheDocument();
    expect(deleteAgent).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));

    await vi.waitFor(() => expect(deleteAgent).toHaveBeenCalledWith('agent-1'));
    await vi.waitFor(() => expect(nav.navigate).toHaveBeenCalledWith('..', undefined));
  });

  it('asks for nothing at all without an agent id', () => {
    render(AgentDetailsPage, {});

    expect(getAgent).not.toHaveBeenCalled();
    expect(getAgentStats).not.toHaveBeenCalled();
  });
});

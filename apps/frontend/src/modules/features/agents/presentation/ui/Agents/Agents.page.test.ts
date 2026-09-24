import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AgentsRepository } from '../../../domain/repository/agents.repository.ts';
import AgentsPage from './Agents.page.svelte';

const agent = (overrides: Record<string, unknown> = {}) => ({
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
  ...overrides,
});

let listAgents: ReturnType<typeof vi.fn>;
let createAgent: ReturnType<typeof vi.fn>;
let deleteAgent: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

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
  listAgents = vi.fn().mockResolvedValue(ScyllaResult.success([agent()]));
  createAgent = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ agent: agent({ id: 'agent-new' }), secret: 'sk-once' }));
  deleteAgent = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    agents: {
      agentsRepository: {
        listAgents,
        createAgent,
        deleteAgent,
        getAgent: vi.fn(),
        getAgentStats: vi.fn(),
      } as unknown as AgentsRepository,
    },
  });

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  grant('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

describe('AgentsPage', () => {
  it('lists the organization’s agents', async () => {
    render(AgentsPage);

    expect(await screen.findByText('runner-1')).toBeInTheDocument();
    expect(listAgents).toHaveBeenCalledWith('org-1');
  });

  it('summarises how many agents are online and offline', async () => {
    listAgents.mockResolvedValue(
      ScyllaResult.success([
        agent({ id: 'a1', connected: true }),
        agent({ id: 'a2', name: 'runner-2', connected: false }),
      ]),
    );
    render(AgentsPage);

    expect(await screen.findByText('online')).toBeInTheDocument();
    expect(screen.getByText('offline')).toBeInTheDocument();
  });

  it('explains the empty state instead of showing a bare list', async () => {
    listAgents.mockResolvedValue(ScyllaResult.success([]));
    render(AgentsPage);

    expect(await screen.findByText('No agents connected')).toBeInTheDocument();
  });

  it('offers to create the first agent when the caller may', async () => {
    listAgents.mockResolvedValue(ScyllaResult.success([]));
    render(AgentsPage);

    expect(await screen.findByRole('button', { name: 'Create your first agent' })).toBeInTheDocument();
  });

  it('does not offer to create without CREATE_AGENT', async () => {
    listAgents.mockResolvedValue(ScyllaResult.success([]));
    grant([Permission.LIST_AGENTS]);
    render(AgentsPage);

    await screen.findByText('No agents connected');
    expect(screen.queryByRole('button', { name: 'Create your first agent' })).not.toBeInTheDocument();
  });

  it('reports a failed load rather than an empty list', async () => {
    listAgents.mockResolvedValue(
      ScyllaResult.error(new ScyllaError('boom', { cause: { code: 'INTERNAL' } })),
    );
    render(AgentsPage);

    expect(await screen.findByText('Error loading agents')).toBeInTheDocument();
  });

  it('creates an agent and reveals its one-time secret', async () => {
    render(AgentsPage);
    await screen.findByText('runner-1');

    await userEvent.click(screen.getAllByRole('button', { name: 'New Agent' })[0]);
    await focusSettled();
    await userEvent.type(await screen.findByLabelText(/name/i), 'runner-9');
    await userEvent.click(screen.getByRole('button', { name: 'Create & reveal secret →' }));

    await vi.waitFor(() => expect(createAgent).toHaveBeenCalledWith('org-1', 'runner-9'));
    // Returned once: the dialog is the only place it shows.
    expect(await screen.findByText('sk-once')).toBeInTheDocument();
  });

  it('refuses to create an agent with a blank name', async () => {
    render(AgentsPage);
    await screen.findByText('runner-1');

    await userEvent.click(screen.getAllByRole('button', { name: 'New Agent' })[0]);
    await focusSettled();
    await userEvent.type(await screen.findByLabelText(/name/i), '   ');
    await userEvent.click(screen.getByRole('button', { name: 'Create & reveal secret →' }));

    expect(createAgent).not.toHaveBeenCalled();
  });

  it('asks for confirmation before deleting, and deletes only on continue', async () => {
    render(AgentsPage);
    await screen.findByText('runner-1');

    await userEvent.click(screen.getByRole('button', { name: 'Delete agent' }));
    expect(await screen.findByText('Delete agent?')).toBeInTheDocument();
    expect(deleteAgent).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await vi.waitFor(() => expect(deleteAgent).toHaveBeenCalledWith('agent-1'));
  });
});

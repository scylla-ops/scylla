import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, usePermissionsStore } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import AgentOutcomesChart from './AgentOutcomesChart.svelte';

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

/** `daysAgo` in ISO, so a fixture always lands inside the window under test. */
const isoDaysAgo = (daysAgo: number): string => {
  const date = new Date();
  date.setDate(date.getDate() - daysAgo);
  return date.toISOString();
};

const stats = (daily: Record<string, unknown>[]) => ({
  completed: 0,
  failed: 0,
  cancelled: 0,
  orphaned: 0,
  daily,
  medianDurationMs: null,
  p95DurationMs: null,
});

const day = (daysAgo: number, counts: Partial<Record<string, number>> = {}) => ({
  day: isoDaysAgo(daysAgo),
  completed: 0,
  failed: 0,
  cancelled: 0,
  orphaned: 0,
  medianDurationMs: null,
  ...counts,
});

let listAgents: ReturnType<typeof vi.fn>;
let getAgentStats: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

/** Series strokes, in draw order — how many curves the chart is showing. */
const drawnSeries = (container: HTMLElement): string[] =>
  [...container.querySelectorAll('path[stroke]')].map(path => path.getAttribute('stroke') ?? '');

beforeEach(() => {
  listAgents = vi.fn().mockResolvedValue(ScyllaResult.success([agent()]));
  getAgentStats = vi
    .fn()
    .mockResolvedValue(
      ScyllaResult.success(stats([day(1, { completed: 3, failed: 1, cancelled: 2 })])),
    );

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    agents: { agentsRepository: { listAgents, getAgentStats } },
  });

  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  } as never);
  usePermissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access: {
            kind: 'restricted',
            permissions: [Permission.LIST_AGENTS, Permission.READ_APP_STATS],
          },
        },
      ],
    },
  });
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  usePermissionsStore.setState({ permissions: null });
});

describe('AgentOutcomesChart', () => {
  it('asks for nothing and explains itself when the organization has no agent', async () => {
    listAgents.mockResolvedValue(ScyllaResult.success([]));
    render(AgentOutcomesChart);

    expect(
      await screen.findByText('No agents found. Connect an agent to see execution history.'),
    ).toBeInTheDocument();
    expect(getAgentStats).not.toHaveBeenCalled();
  });

  it('reads the first agent’s stats without being asked to pick one', async () => {
    render(AgentOutcomesChart);

    await screen.findByRole('img', { name: 'Agent Outcomes' });
    expect(getAgentStats).toHaveBeenCalledWith('agent-1');
  });

  it('switches to the agent the reader picks', async () => {
    const user = userEvent.setup();
    listAgents.mockResolvedValue(
      ScyllaResult.success([agent(), agent({ id: 'agent-2', name: 'runner-2' })]),
    );
    render(AgentOutcomesChart);

    await user.click(await screen.findByRole('button', { name: 'runner-2' }));

    expect(getAgentStats).toHaveBeenCalledWith('agent-2');
  });

  it('draws the three series, with a legend, until a filter narrows it to one', async () => {
    const user = userEvent.setup();
    const { container } = render(AgentOutcomesChart);

    await screen.findByRole('img', { name: 'Agent Outcomes' });
    expect(drawnSeries(container)).toHaveLength(3);

    await user.click(screen.getByRole('button', { name: 'failed' }));

    expect(drawnSeries(container)).toEqual(['var(--destructive)']);
  });

  it('says so rather than drawing an empty frame when nothing finished in the window', async () => {
    getAgentStats.mockResolvedValue(ScyllaResult.success(stats([])));
    render(AgentOutcomesChart);

    expect(await screen.findByText('No finished jobs in this window.')).toBeInTheDocument();
    expect(screen.queryByRole('img', { name: 'Agent Outcomes' })).not.toBeInTheDocument();
  });

  it('narrows the window to the range picked, which can empty it', async () => {
    const user = userEvent.setup();
    // Outside 7 days, inside 14: the range button is what decides.
    getAgentStats.mockResolvedValue(ScyllaResult.success(stats([day(9, { completed: 2 })])));
    render(AgentOutcomesChart);

    await screen.findByRole('img', { name: 'Agent Outcomes' });

    await user.click(screen.getByRole('button', { name: '7d' }));

    expect(screen.getByText('No finished jobs in this window.')).toBeInTheDocument();
  });

  it('labels the y axis in whole runs, from zero to above the peak', async () => {
    render(AgentOutcomesChart);

    await screen.findByRole('img', { name: 'Agent Outcomes' });
    // Peak is 3 completed on one day, so the axis runs 0..3 in steps of one —
    // never "1.5 runs", which is what `allowDecimals={false}` used to buy.
    for (const tick of ['0', '1', '2', '3']) {
      expect(screen.getByText(tick)).toBeInTheDocument();
    }
  });
});

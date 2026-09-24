import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import type { JobNodeExecution } from '../../../../domain/structs/job.struct.ts';
import JobTimeline from './JobTimeline.svelte';

const node = (overrides: Partial<JobNodeExecution> = {}): JobNodeExecution => ({
  id: 'build',
  state: 'completed',
  ...overrides,
});

const many = (count: number, state = 'completed') =>
  Array.from({ length: count }, (_, index) => node({ id: `node-${index}`, state }));

describe('JobTimeline', () => {
  it('shows a "no nodes" empty state', () => {
    render(JobTimeline, { nodeExecutions: [] });
    expect(screen.getByText('No nodes')).toBeInTheDocument();
  });

  it('below the collapse threshold, shows one segment per node', () => {
    render(JobTimeline, { nodeExecutions: many(3), onSelectNode: vi.fn() });

    expect(screen.getByRole('button', { name: 'Node node-0' })).toBeInTheDocument();
    expect(screen.getAllByRole('button')).toHaveLength(3);
  });

  it('past the threshold, groups nodes into one segment per status', () => {
    render(JobTimeline, {
      nodeExecutions: [...many(10, 'completed'), ...many(2, 'failed')],
      onSelectNode: vi.fn(),
    });

    expect(screen.getAllByRole('button')).toHaveLength(2);
    expect(screen.getByRole('button', { name: '10 Success nodes' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '2 Failed nodes' })).toBeInTheDocument();
  });

  it('shows the node count on a group wide enough to hold it', () => {
    render(JobTimeline, { nodeExecutions: many(11), onSelectNode: vi.fn() });
    expect(screen.getByText('11')).toBeInTheDocument();
  });

  it('clicking a node segment reports that node', async () => {
    const onSelectNode = vi.fn();
    render(JobTimeline, { nodeExecutions: many(3), onSelectNode });

    await userEvent.click(screen.getByRole('button', { name: 'Node node-1' }));

    expect(onSelectNode).toHaveBeenCalledWith('node-1');
  });

  it('a grouped segment stands for several nodes, so it reports none', async () => {
    const onSelectNode = vi.fn();
    render(JobTimeline, { nodeExecutions: many(11), onSelectNode });

    await userEvent.click(screen.getByRole('button', { name: '11 Success nodes' }));

    expect(onSelectNode).toHaveBeenCalledWith();
  });

  it('renders plain bars, not buttons, when there is nowhere to select to', () => {
    render(JobTimeline, { nodeExecutions: many(3) });
    expect(screen.queryAllByRole('button')).toHaveLength(0);
  });

  it("shows a finished node's duration in its tooltip", async () => {
    render(JobTimeline, {
      nodeExecutions: [
        node({
          startedAt: '2026-01-01T00:00:00.000Z',
          finishedAt: '2026-01-01T00:00:30.000Z',
        }),
      ],
      onSelectNode: vi.fn(),
    });

    await userEvent.hover(screen.getByRole('button', { name: 'Node build' }));
    const tooltip = await findFloating('tooltip');

    expect(tooltip).toHaveTextContent('Duration: 30s');
  });

  it('shows no duration line for a node that has not started', async () => {
    render(JobTimeline, {
      nodeExecutions: [node({ state: 'pending' })],
      onSelectNode: vi.fn(),
    });

    await userEvent.hover(screen.getByRole('button', { name: 'Node build' }));
    const tooltip = await findFloating('tooltip');

    expect(tooltip).not.toHaveTextContent('Duration:');
    expect(tooltip).toHaveTextContent('State: Pending');
  });
});

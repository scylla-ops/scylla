import { describe, it, expect, vi, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithI18n } from '@/test/render.tsx';
import { JobTimeline } from './JobTimeline';
import type { JobNodeExecution } from '@/modules/features/jobs/domain/structs/job.struct.ts';

const node = (overrides: Partial<JobNodeExecution> = {}): JobNodeExecution => ({
  id: 'checkout',
  state: 'completed',
  ...overrides,
});

describe('JobTimeline', () => {
  it('shows a "no nodes" empty state', () => {
    renderWithI18n(<JobTimeline nodeExecutions={[]} />);
    expect(screen.getByText('No nodes')).toBeInTheDocument();
  });

  it('below the collapse threshold, shows one segment per node', () => {
    const { container } = renderWithI18n(
      <JobTimeline
        nodeExecutions={[node({ id: 'a' }), node({ id: 'b' }), node({ id: 'c', state: 'failed' })]}
      />,
    );
    // StatusBar's own rendering is covered elsewhere - this only checks the
    // right number of nodes reached it, not one grouped-by-status segment.
    expect(container.querySelectorAll('.flex-1.min-w-\\[2px\\]')).toHaveLength(3);
  });

  it('past the collapse threshold, groups nodes into one segment per status', () => {
    const nodes = [
      ...Array.from({ length: 8 }, (_, i) => node({ id: `ok-${i}`, state: 'completed' })),
      ...Array.from({ length: 3 }, (_, i) => node({ id: `bad-${i}`, state: 'failed' })),
    ];
    const { container } = renderWithI18n(<JobTimeline nodeExecutions={nodes} />);

    // Two groups (completed, failed), not eleven individual segments.
    const segments = container.querySelectorAll('.h-full.rounded-sm');
    expect(segments).toHaveLength(2);
  });

  it('shows the node count on a group wide enough to hold it', () => {
    const nodes = Array.from({ length: 11 }, (_, i) => node({ id: `n-${i}` }));
    renderWithI18n(<JobTimeline nodeExecutions={nodes} />);
    // A single group spans 100% of the bar, comfortably over the 8% cutoff.
    expect(screen.getByText('11')).toBeInTheDocument();
  });

  it('clicking a node segment reports that node', async () => {
    const onSelectNode = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <JobTimeline nodeExecutions={[node({ id: 'build' })]} onSelectNode={onSelectNode} />,
    );

    await user.click(screen.getByRole('button', { name: 'Node build' }));
    expect(onSelectNode).toHaveBeenCalledWith('build');
  });

  it('a grouped segment stands for several nodes, so it reports none', async () => {
    const onSelectNode = vi.fn();
    const user = userEvent.setup();
    const nodes = Array.from({ length: 11 }, (_, i) => node({ id: `n-${i}` }));
    renderWithI18n(<JobTimeline nodeExecutions={nodes} onSelectNode={onSelectNode} />);

    await user.click(screen.getByRole('button', { name: /11/ }));
    expect(onSelectNode).toHaveBeenCalledWith();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows a finished node's duration in its tooltip", async () => {
    const user = userEvent.setup();
    const { container } = renderWithI18n(
      <JobTimeline
        nodeExecutions={[
          node({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:01:05.000Z' }),
        ]}
      />,
    );

    await user.hover(container.querySelector('.flex-1.min-w-\\[2px\\]')!);
    await waitFor(() => expect(screen.getAllByText('Duration: 1m 5s').length).toBeGreaterThan(0));
  });

  it('shows the elapsed-so-far duration for a still-running node', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.setSystemTime(new Date('2026-01-01T00:00:30.000Z'));
    const user = userEvent.setup();
    const { container } = renderWithI18n(
      <JobTimeline
        nodeExecutions={[
          node({ state: 'running', startedAt: '2026-01-01T00:00:00.000Z', finishedAt: undefined }),
        ]}
      />,
    );

    await user.hover(container.querySelector('.flex-1.min-w-\\[2px\\]')!);
    await waitFor(() => expect(screen.getAllByText('Duration: 30s').length).toBeGreaterThan(0));
  });

  it('shows no duration line for a node that has not started', async () => {
    const user = userEvent.setup();
    const { container } = renderWithI18n(<JobTimeline nodeExecutions={[node({ state: 'pending' })]} />);

    await user.hover(container.querySelector('.flex-1.min-w-\\[2px\\]')!);
    await waitFor(() => expect(screen.getAllByText(/^State:/).length).toBeGreaterThan(0));
    expect(screen.queryByText(/^Duration:/)).not.toBeInTheDocument();
  });
});

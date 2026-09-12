import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { JobTimeline } from './JobTimeline';
import type { JobNodeExecution } from '@/modules/features/jobs/domain/structs/job.struct.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const node = (overrides: Partial<JobNodeExecution> = {}): JobNodeExecution => ({
  id: 'checkout',
  state: 'completed',
  ...overrides,
});

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
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
});

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { JobNodesList } from './JobNodesList';
import type { JobNodeExecution } from '@/modules/features/jobs/domain/structs/job.struct.ts';

vi.mock('@/modules/features/jobs/presentation/ui/jobs-table/jobs-log/JobLogDisplay.tsx', () => ({
  JobLogDisplay: ({ jobId, nodeId }: { jobId: string; nodeId: string }) => (
    <div data-testid='job-log-display'>
      logs for {jobId}/{nodeId}
    </div>
  ),
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const node = (overrides: Partial<JobNodeExecution> = {}): JobNodeExecution => ({
  id: 'checkout',
  state: 'completed',
  ...overrides,
});

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.useRealTimers();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

describe('JobNodesList', () => {
  it('renders nothing while collapsed', () => {
    const { container } = renderWithI18n(
      <JobNodesList jobId='job-1' nodeExecutions={[node()]} isExpanded={false} onCollapse={vi.fn()} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('shows the node count in the header, once expanded', () => {
    renderWithI18n(
      <JobNodesList
        jobId='job-1'
        nodeExecutions={[node({ id: 'a' }), node({ id: 'b' })]}
        isExpanded
        onCollapse={vi.fn()}
      />,
    );
    expect(screen.getByText(/Node Executions \(2\)/)).toBeInTheDocument();
  });

  it('the collapse button calls onCollapse', async () => {
    const onCollapse = vi.fn();
    const user = userEvent.setup();
    const { container } = renderWithI18n(
      <JobNodesList jobId='job-1' nodeExecutions={[node()]} isExpanded onCollapse={onCollapse} />,
    );

    // IconButton's tooltip text isn't its accessible name - the header's only
    // button is this one, found directly rather than by role+name.
    await user.click(container.querySelector('h4 button')!);
    expect(onCollapse).toHaveBeenCalled();
  });

  it('clicking a node toggles its own logs open, independently of the others', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <JobNodesList
        jobId='job-1'
        nodeExecutions={[node({ id: 'checkout' }), node({ id: 'test' })]}
        isExpanded
        onCollapse={vi.fn()}
      />,
    );

    expect(screen.queryByTestId('job-log-display')).not.toBeInTheDocument();

    await user.click(screen.getByText('checkout'));
    expect(screen.getByText('logs for job-1/checkout')).toBeInTheDocument();
    expect(screen.queryByText('logs for job-1/test')).not.toBeInTheDocument();

    // Collapsing it again hides the logs.
    await user.click(screen.getByText('checkout'));
    expect(screen.queryByTestId('job-log-display')).not.toBeInTheDocument();
  });

  describe('duration', () => {
    it('shows a dash for a node that has not started', () => {
      renderWithI18n(
        <JobNodesList
          jobId='job-1'
          nodeExecutions={[node({ startedAt: undefined, finishedAt: undefined })]}
          isExpanded
          onCollapse={vi.fn()}
        />,
      );
      expect(screen.getByText('-')).toBeInTheDocument();
    });

    it('formats seconds-only for a short run', () => {
      renderWithI18n(
        <JobNodesList
          jobId='job-1'
          nodeExecutions={[
            node({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:00:45.000Z' }),
          ]}
          isExpanded
          onCollapse={vi.fn()}
        />,
      );
      expect(screen.getByText('45s')).toBeInTheDocument();
    });

    it('formats minutes+seconds once past a minute', () => {
      renderWithI18n(
        <JobNodesList
          jobId='job-1'
          nodeExecutions={[
            node({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:02:05.000Z' }),
          ]}
          isExpanded
          onCollapse={vi.fn()}
        />,
      );
      expect(screen.getByText('2m 5s')).toBeInTheDocument();
    });

    it('formats hours+minutes once past an hour', () => {
      renderWithI18n(
        <JobNodesList
          jobId='job-1'
          nodeExecutions={[
            node({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T01:30:00.000Z' }),
          ]}
          isExpanded
          onCollapse={vi.fn()}
        />,
      );
      expect(screen.getByText('1h 30m')).toBeInTheDocument();
    });

    it('a still-running node (no finishedAt) measures against now', () => {
      vi.useFakeTimers();
      vi.setSystemTime(new Date('2026-01-01T00:01:30.000Z'));

      renderWithI18n(
        <JobNodesList
          jobId='job-1'
          nodeExecutions={[node({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: undefined })]}
          isExpanded
          onCollapse={vi.fn()}
        />,
      );
      expect(screen.getByText('1m 30s')).toBeInTheDocument();

      vi.useRealTimers();
    });
  });

  it('shows the node id and its status label', () => {
    renderWithI18n(
      <JobNodesList jobId='job-1' nodeExecutions={[node({ id: 'deploy', state: 'failed' })]} isExpanded onCollapse={vi.fn()} />,
    );
    expect(screen.getByText('deploy')).toBeInTheDocument();
    expect(screen.getByText('Failed')).toBeInTheDocument();
  });
});

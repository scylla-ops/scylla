import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { renderWithProviders } from '@/test/render.tsx';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { JobDetailsPage } from './JobDetails.page';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';

vi.mock('@/modules/features/jobs/presentation/ui/jobs-log/JobLogDisplay.tsx', () => ({
  JobLogDisplay: ({
    jobId,
    nodeId,
    maxHeight,
  }: {
    jobId: string;
    nodeId?: string;
    maxHeight?: number;
  }) => (
    <div data-testid='job-log-display' data-max-height={maxHeight}>
      logs for {jobId}/{nodeId ?? 'whole job'}
    </div>
  ),
}));

/**
 * The log column measures itself to decide how tall the whole job's log may
 * grow, and jsdom lays nothing out — so the suite-wide inert stub is replaced
 * here by one a test can report a real height through.
 */
class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  callback: ResizeObserverCallback;
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
    ResizeObserverMock.instances.push(this);
  }

  fire(height: number) {
    this.callback([{ contentRect: { height } } as unknown as ResizeObserverEntry], this);
  }
}

const giveTheLogColumn = (height: number) =>
  act(() => ResizeObserverMock.instances.forEach(observer => observer.fire(height)));

const logHeights = () =>
  screen.queryAllByTestId('job-log-display').map(panel => panel.getAttribute('data-max-height'));

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [
    { id: 'build', state: 'completed' },
    { id: 'test', state: 'failed' },
  ],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
  startedAt: '2026-01-01T00:00:00.000Z',
  finishedAt: '2026-01-01T00:00:45.000Z',
  ...overrides,
});

const repositoryReturning = (result: unknown): JobsRepository =>
  ({
    getById: vi.fn().mockResolvedValue(result),
  }) as unknown as JobsRepository;

const renderPage = (repository: JobsRepository, search = '') =>
  renderWithProviders(
    <MemoryRouter initialEntries={[`/o/projects/p/pipelines/pipeline-1/jobs/job-1${search}`]}>
      <Routes>
        <Route
          path='/o/projects/p/pipelines/:pipelineId/jobs/:jobId'
          element={<JobDetailsPage />}
        />
      </Routes>
    </MemoryRouter>,
    { registry: { jobs: { jobsRepository: repository } } },
  );

const openPanels = () => screen.queryAllByTestId('job-log-display').map(panel => panel.textContent);

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });
});

describe('JobDetailsPage', () => {
  it('shows the job status, id and execution times', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    expect(await screen.findByText('Success')).toBeInTheDocument();
    expect(screen.getByText('job-1')).toBeInTheDocument();
    expect(screen.getByText('45s')).toBeInTheDocument();
  });

  it('lists one log entry per node execution, plus the whole job', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    const nodes = within(await screen.findByRole('navigation', { name: 'Node executions' }));
    expect(nodes.getByRole('button', { name: 'Whole job' })).toBeInTheDocument();
    expect(nodes.getByRole('button', { name: /build/ })).toBeInTheDocument();
    expect(nodes.getByRole('button', { name: /test/ })).toBeInTheDocument();
  });

  it('defaults to the whole job when the URL names no node', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    expect(await screen.findByTestId('job-log-display')).toHaveTextContent(
      'logs for job-1/whole job',
    );
  });

  it("opens straight on a node's logs when the URL names one", async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=test');

    expect(await screen.findByTestId('job-log-display')).toHaveTextContent('logs for job-1/test');
  });

  it('falls back to the whole job for a node id no execution matches', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=ghost');

    expect(await screen.findByTestId('job-log-display')).toHaveTextContent(
      'logs for job-1/whole job',
    );
  });

  it('streams several nodes at once when the URL names them', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build,test');

    await waitFor(() => expect(screen.queryAllByTestId('job-log-display')).toHaveLength(2));
    expect(openPanels()).toEqual(['logs for job-1/build', 'logs for job-1/test']);
  });

  it('replaces the whole job with the first node picked, rather than adding to it', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    const nodes = within(await screen.findByRole('navigation', { name: 'Node executions' }));
    await user.click(nodes.getByRole('button', { name: /build/ }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/build']));
  });

  it('opens a second node without closing the first', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build');

    const nodes = within(await screen.findByRole('navigation', { name: 'Node executions' }));
    await user.click(nodes.getByRole('button', { name: /test/ }));

    await waitFor(() => expect(screen.queryAllByTestId('job-log-display')).toHaveLength(2));
    expect(openPanels()).toEqual(['logs for job-1/build', 'logs for job-1/test']);
  });

  it("closes a node's logs when its entry is picked again, unmounting that view", async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build,test');

    const nodes = within(await screen.findByRole('navigation', { name: 'Node executions' }));
    await user.click(nodes.getByRole('button', { name: /build/ }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/test']));
  });

  it('comes back to the whole job when the last node panel is closed', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build');

    await user.click(await screen.findByRole('button', { name: 'Close the logs for build' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/whole job']));
  });

  it('brings the whole job back from its own entry, dropping the node panels', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build,test');

    const nodes = within(await screen.findByRole('navigation', { name: 'Node executions' }));
    await user.click(nodes.getByRole('button', { name: 'Whole job' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/whole job']));
  });

  it('gives the whole job no way to be closed, being what the page falls back to', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    await screen.findByTestId('job-log-display');
    expect(
      screen.queryByRole('button', { name: 'Close the logs for Whole job' }),
    ).not.toBeInTheDocument();
  });

  it('marks the entries of the panels that are open', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build');

    const nodes = within(await screen.findByRole('navigation', { name: 'Node executions' }));
    expect(nodes.getByRole('button', { name: /build/ })).toHaveAttribute('aria-pressed', 'true');
    expect(nodes.getByRole('button', { name: /test/ })).toHaveAttribute('aria-pressed', 'false');
    expect(nodes.getByRole('button', { name: 'Whole job' })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
  });

  it('opens the node picked in the timeline alone, dropping the panels open beside it', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=test');

    await user.click(await screen.findByRole('button', { name: 'Node build' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/build']));
  });

  it('comes back to the whole job from a timeline segment standing for several nodes', async () => {
    const user = userEvent.setup();
    const nodeExecutions = Array.from({ length: 11 }, (_, index) => ({
      id: `node-${index}`,
      state: 'completed' as const,
    }));
    renderPage(repositoryReturning(ScyllaResult.success(job({ nodeExecutions }))), '?nodes=node-3');

    await user.click(await screen.findByRole('button', { name: '11 Success nodes' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/whole job']));
  });

  it('gives the whole job every pixel the column has', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())));
    await screen.findByTestId('job-log-display');

    giveTheLogColumn(800);

    expect(logHeights()).toEqual(['764']);
  });

  it('keeps every node panel at the same readable height, scrolling the column instead', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())), '?nodes=build,test');
    await waitFor(() => expect(screen.queryAllByTestId('job-log-display')).toHaveLength(2));

    giveTheLogColumn(800);

    expect(logHeights()).toEqual(['448', '448']);
  });

  it('stops shrinking the whole job at a readable height on a short window', async () => {
    renderPage(repositoryReturning(ScyllaResult.success(job())));
    await screen.findByTestId('job-log-display');

    giveTheLogColumn(100);

    expect(logHeights()).toEqual(['192']);
  });

  it('hides the logs, keeping the job itself, without READ_JOB_LOGS', async () => {
    usePermissionsStore.setState({
      permissions: {
        scopes: [
          {
            scope: PermissionScope.SYSTEM,
            scopeId: '',
            access: { kind: 'restricted', permissions: [Permission.READ_JOB] },
          },
        ],
      },
    });
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    expect(await screen.findByText('Success')).toBeInTheDocument();
    expect(screen.queryByTestId('job-log-display')).not.toBeInTheDocument();
    expect(
      screen.getByText("You don't have permission to view this job's logs"),
    ).toBeInTheDocument();
  });

  it('reports an error instead of an empty page when the job cannot be read', async () => {
    renderPage(repositoryReturning(ScyllaResult.error(new ScyllaError('boom'))));

    expect(await screen.findByText('Unable to load this job')).toBeInTheDocument();
  });
});

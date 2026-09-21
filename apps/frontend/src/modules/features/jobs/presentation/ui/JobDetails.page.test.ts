import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { flushSync } from 'svelte';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { Permission, PermissionScope, usePermissionsStore } from '@platform/authz';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobEntity } from '../../domain/entities/job.entity.ts';
import type { JobsRepository } from '../../domain/repository/jobs.repository.ts';
import JobDetailsPage from './JobDetails.page.svelte';

/**
 * The real viewer opens a CodeMirror instance per panel and a live gRPC stream
 * behind it. What this page is responsible for is *which* panels exist and how
 * tall they may grow, so the viewer is stubbed down to those two facts.
 */
vi.mock('./jobs-log/JobLogDisplay.svelte', async () => ({
  default: (await import('./jobs-log/JobLogDisplay.stub.svelte')).default,
}));

/**
 * The log column measures itself to decide how tall the whole job's log may
 * grow, and jsdom lays nothing out — so the suite-wide inert stub is replaced
 * here by one a test can report a real height through.
 */
class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  callback: (entries: Array<{ contentRect: { height: number } }>) => void;
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();

  constructor(callback: (entries: Array<{ contentRect: { height: number } }>) => void) {
    this.callback = callback;
    ResizeObserverMock.instances.push(this);
  }

  fire(height: number) {
    this.callback([{ contentRect: { height } }]);
  }
}

const giveTheLogColumn = (height: number) => {
  ResizeObserverMock.instances.forEach(observer => observer.fire(height));
  flushSync();
};

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

const openPanels = () =>
  screen.queryAllByTestId('job-log-display').map(panel => panel.textContent?.trim());

const logHeights = () =>
  screen.queryAllByTestId('job-log-display').map(panel => panel.getAttribute('data-max-height'));

let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: (() => void) | null = null;
let navigator: ReturnType<typeof installTestNavigator>;

const grantAll = () =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const renderPage = async (result: unknown, search = '') => {
  restoreRegistry = withRegistry({
    jobs: { jobsRepository: { getById: vi.fn().mockResolvedValue(result) } as unknown as JobsRepository },
  });
  navigator = installTestNavigator({
    pathname: '/acme/projects/p1/pipelines/pipeline-1/jobs/job-1',
    search,
  });

  const rendered = render(JobDetailsPage, { jobId: 'job-1' });
  await screen.findByRole('heading', { name: 'Job' });
  return rendered;
};

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
  cache = withQueryClient();
  grantAll();
});

afterEach(() => {
  cache.restore();
  restoreRegistry?.();
  restoreRegistry = null;
  navigator?.restore();
  usePermissionsStore.setState({ permissions: null });
});

describe('JobDetailsPage', () => {
  it('shows the job status and id', async () => {
    await renderPage(ScyllaResult.success(job()));

    expect(screen.getByText('Success')).toBeInTheDocument();
    expect(screen.getByText('job-1')).toBeInTheDocument();
  });

  it('defaults to the whole job when the URL names no node', async () => {
    await renderPage(ScyllaResult.success(job()));
    expect(openPanels()).toEqual(['logs for job-1/whole job']);
  });

  it("opens straight on a node's logs when the URL names one", async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=test');
    expect(openPanels()).toEqual(['logs for job-1/test']);
  });

  it('falls back to the whole job for a node id no execution matches', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=ghost');
    expect(openPanels()).toEqual(['logs for job-1/whole job']);
  });

  it('streams several nodes at once when the URL names them', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build,test');
    expect(openPanels()).toEqual(['logs for job-1/build', 'logs for job-1/test']);
  });

  it('replaces the whole job with the first node picked, rather than adding to it', async () => {
    await renderPage(ScyllaResult.success(job()));

    await userEvent.click(screen.getByRole('button', { name: /^build/ }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/build']));
  });

  it('opens a second node without closing the first', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build');

    await userEvent.click(screen.getByRole('button', { name: /^test/ }));

    await waitFor(() =>
      expect(openPanels()).toEqual(['logs for job-1/build', 'logs for job-1/test']),
    );
  });

  it("closes a node's logs when its entry is picked again, unmounting that view", async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build,test');

    await userEvent.click(screen.getByRole('button', { name: /^build/ }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/test']));
  });

  it('comes back to the whole job when the last node panel is closed', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build');

    await userEvent.click(screen.getByRole('button', { name: 'Close the logs for build' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/whole job']));
  });

  it('brings the whole job back from its own entry, dropping the node panels', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build,test');

    await userEvent.click(screen.getByRole('button', { name: 'Whole job' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/whole job']));
  });

  it('gives the whole job no way to be closed, being what the page falls back to', async () => {
    await renderPage(ScyllaResult.success(job()));

    expect(screen.queryByRole('button', { name: /^Close the logs/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^Collapse the logs/ })).not.toBeInTheDocument();
  });

  it('collapses a node panel without unmounting its log, so the stream keeps running', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build');

    await userEvent.click(screen.getByRole('button', { name: 'Collapse the logs for build' }));

    expect(openPanels()).toEqual(['logs for job-1/build']);
    expect(screen.getByRole('button', { name: 'Expand the logs for build' })).toBeInTheDocument();
  });

  it('collapses each node panel independently', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build,test');

    await userEvent.click(screen.getByRole('button', { name: 'Collapse the logs for build' }));

    expect(screen.getByRole('button', { name: 'Expand the logs for build' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Collapse the logs for test' })).toBeInTheDocument();
  });

  it('marks the entries of the panels that are open', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build');

    expect(screen.getByRole('button', { name: /^build/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(screen.getByRole('button', { name: /^test/ })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
  });

  it('opens the node picked in the timeline alone, dropping the panels open beside it', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build,test');

    await userEvent.click(screen.getByRole('button', { name: 'Node build' }));

    await waitFor(() => expect(openPanels()).toEqual(['logs for job-1/build']));
  });

  it('gives the whole job every pixel the column has', async () => {
    await renderPage(ScyllaResult.success(job()));

    giveTheLogColumn(900);

    // 900 less the panel's own 36px header and its 2px of border.
    await waitFor(() => expect(logHeights()).toEqual(['862']));
  });

  it('stops shrinking the whole job at a readable height on a short window', async () => {
    await renderPage(ScyllaResult.success(job()));

    giveTheLogColumn(100);

    await waitFor(() => expect(logHeights()).toEqual(['192']));
  });

  it('keeps every node panel at the same readable height, scrolling the column instead', async () => {
    await renderPage(ScyllaResult.success(job()), '?nodes=build,test');

    giveTheLogColumn(300);

    await waitFor(() => expect(logHeights()).toEqual(['448', '448']));
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

    await renderPage(ScyllaResult.success(job()));

    expect(screen.getByText("You don't have permission to view this job's logs")).toBeInTheDocument();
    expect(openPanels()).toEqual([]);
    // The job itself is still there — only its output is withheld.
    expect(screen.getByText('Success')).toBeInTheDocument();
  });

  it('reports an error instead of an empty page when the job cannot be read', async () => {
    restoreRegistry = withRegistry({
      jobs: {
        jobsRepository: {
          getById: vi.fn().mockResolvedValue(ScyllaResult.error(new ScyllaError('boom'))),
        } as unknown as JobsRepository,
      },
    });
    navigator = installTestNavigator({ pathname: '/acme/jobs/job-1' });

    render(JobDetailsPage, { jobId: 'job-1' });

    expect(await screen.findByText('Unable to load this job')).toBeInTheDocument();
  });

  it('refuses to query at all without a job id', async () => {
    const getById = vi.fn();
    restoreRegistry = withRegistry({
      jobs: { jobsRepository: { getById } as unknown as JobsRepository },
    });
    navigator = installTestNavigator({ pathname: '/acme/jobs' });

    render(JobDetailsPage, {});

    expect(await screen.findByText('Job ID is missing')).toBeInTheDocument();
    expect(getById).not.toHaveBeenCalled();
  });
});

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { renderWithProviders } from '@/test/render.tsx';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { JobDetailsPage } from './JobDetails.page';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';

vi.mock('@/modules/features/jobs/presentation/ui/jobs-log/JobLogDisplay.tsx', () => ({
  JobLogDisplay: ({ jobId, nodeId }: { jobId: string; nodeId?: string }) => (
    <div data-testid='job-log-display'>
      logs for {jobId}/{nodeId ?? 'whole job'}
    </div>
  ),
}));

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

  it('closes the whole job panel from its own header', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    await user.click(await screen.findByRole('button', { name: 'Close the logs for Whole job' }));

    await waitFor(() => expect(openPanels()).toEqual([]));
    expect(
      screen.getByText('No logs open — pick the whole job or a node to read its output'),
    ).toBeInTheDocument();
  });

  it('opening a node from the timeline leaves the panels already open alone', async () => {
    const user = userEvent.setup();
    renderPage(repositoryReturning(ScyllaResult.success(job())));

    await user.click(await screen.findByRole('button', { name: 'Node build' }));

    await waitFor(() =>
      expect(openPanels()).toEqual(['logs for job-1/whole job', 'logs for job-1/build']),
    );
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

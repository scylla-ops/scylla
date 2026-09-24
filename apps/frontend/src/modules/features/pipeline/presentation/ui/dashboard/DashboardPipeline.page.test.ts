import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import DashboardPipelinePage from './DashboardPipeline.page.svelte';

vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), warning: vi.fn(), error: vi.fn() } }));

const pipelines = [
  { id: 'pipeline-1', projectId: 'project-1', name: 'nightly', nodeCount: 2, createdAt: '2026-09-01T10:00:00Z', updatedAt: '2026-09-01T10:00:00Z' },
  { id: 'pipeline-2', projectId: 'project-1', name: 'release', nodeCount: 1, createdAt: '2026-09-02T10:00:00Z', updatedAt: '2026-09-02T10:00:00Z' },
];

const job = {
  id: 'job-7',
  pipelineId: 'pipeline-1',
  status: 'success',
  nodeExecutions: [],
  createdAt: '2026-09-03T10:00:00Z',
  updatedAt: '2026-09-03T10:05:00Z',
  startedAt: '2026-09-03T10:00:00Z',
  finishedAt: '2026-09-03T10:05:00Z',
};

const page = (items: typeof pipelines) => ({
  items,
  pagination: { totalCount: items.length, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false },
});

let teardown: Array<() => void> = [];
let navigator: ReturnType<typeof installTestNavigator>;
let run: ReturnType<typeof vi.fn>;

const grantEverything = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const setUp = (getMetadataByProjectId: () => Promise<unknown>) => {
  run = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const cache = withQueryClient();
  teardown = [
    cache.restore,
    withRegistry({
      pipeline: { pipelineRepository: { getMetadataByProjectId, run } },
      jobs: {
        jobsRepository: {
          getByPipelineId: (pipelineId: string) =>
            Promise.resolve(
              ScyllaResult.success({ items: pipelineId === 'pipeline-1' ? [job] : [] }),
            ),
        },
      },
      agents: { agentsRepository: { listAgents: () => Promise.resolve(ScyllaResult.success([])) } },
    }),
  ];
  return render(DashboardPipelinePage, { projectId: 'project-1' });
};

beforeEach(() => {
  navigator = installTestNavigator({ pathname: '/acme/projects/project-1' });
  grantEverything();
  selectionStore.setState({ selectedIds: {} });
  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'web' },
  });
});

afterEach(() => {
  teardown.forEach(restore => restore());
  navigator.restore();
});

describe('DashboardPipelinePage', () => {
  it('lists the pipelines of the project with their count', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page(pipelines))));

    expect(await screen.findByText('nightly')).toBeInTheDocument();
    expect(screen.getByText('release')).toBeInTheDocument();
  });

  it('says so when the project has no pipeline', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page([]))));

    expect(await screen.findByText('No pipeline found')).toBeInTheDocument();
  });

  it('shows the error of the backend when the list fails', async () => {
    setUp(() => Promise.resolve(ScyllaResult.error(new ScyllaError('backend is down'))));

    expect(await screen.findByText('Error')).toBeInTheDocument();
    expect(screen.queryByText('nightly')).not.toBeInTheDocument();
  });

  it('opens the last run of a pipeline', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page(pipelines))));

    await userEvent.click(await screen.findByRole('button', { name: 'Open the last run' }));

    expect(navigator.navigate).toHaveBeenCalledWith(
      '/acme/projects/project-1/pipelines/pipeline-1/jobs/job-7',
      undefined,
    );
  });

  it('runs a pipeline from its row', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page(pipelines))));
    await screen.findByText('nightly');

    const [runNightly] = screen.getAllByRole('button', { name: 'Run' });
    await userEvent.click(runNightly);

    await waitFor(() => expect(run).toHaveBeenCalledWith('pipeline-1'));
  });

  it('opens the editor, the jobs and the triggers of a pipeline', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page(pipelines))));
    await screen.findByText('nightly');

    await userEvent.click(screen.getAllByRole('button', { name: 'Edit pipeline' })[0]);
    await userEvent.click(screen.getAllByRole('button', { name: 'View Jobs' })[0]);
    await userEvent.click(screen.getAllByRole('button', { name: 'Triggers' })[0]);

    expect(navigator.navigate).toHaveBeenCalledWith(
      '/acme/projects/project-1/edit/pipeline-1',
      undefined,
    );
    expect(navigator.navigate).toHaveBeenCalledWith(
      '/acme/projects/project-1/pipelines/pipeline-1/jobs',
      undefined,
    );
    expect(navigator.navigate).toHaveBeenCalledWith(
      '/acme/projects/project-1/pipelines/pipeline-1/triggers',
      undefined,
    );
  });

  it('opens the members and the secrets of the project from the header', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page(pipelines))));
    await screen.findByText('nightly');

    await userEvent.click(screen.getByRole('button', { name: 'Members' }));
    await userEvent.click(screen.getByRole('button', { name: 'Secrets' }));

    expect(navigator.navigate).toHaveBeenCalledWith('/acme/projects/project-1/members', {});
    expect(navigator.navigate).toHaveBeenCalledWith('/acme/projects/project-1/secrets', {});
  });

  it('selects a pipeline when the user clicks its row', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(page(pipelines))));

    await userEvent.click(await screen.findByText('release'));

    expect(selectionStore.getState().selectedIds.pipelines).toEqual(['pipeline-2']);
  });
});

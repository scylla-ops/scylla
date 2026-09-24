import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobsRepository } from '../../../domain/repository/jobs.repository.ts';
import JobsHeader from './JobsHeader.svelte';

const deleteById = vi.fn();

const grant = (permissions: Permission[] | 'all') =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access:
            permissions === 'all'
              ? { kind: 'fullControl' }
              : { kind: 'restricted', permissions },
        },
      ],
    },
  });

let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

beforeEach(() => {
  deleteById.mockReset().mockResolvedValue(ScyllaResult.success(undefined));
  cache = withQueryClient();
  restoreRegistry = withRegistry({
    jobs: { jobsRepository: { deleteById } as unknown as JobsRepository },
  });
  selectionStore.setState({ selectedIds: {} });
  grant('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

const props = (overrides: Record<string, unknown> = {}) => ({
  numberOfJobs: 3,
  jobIds: ['job-1', 'job-2', 'job-3'],
  pipelineId: 'pipeline-1',
  onRefresh: vi.fn(),
  ...overrides,
});

describe('JobsHeader', () => {
  it('shows the job count and the pipeline id', () => {
    render(JobsHeader, props());

    expect(screen.getByText('3')).toBeInTheDocument();
    expect(screen.getByText('pipeline-1')).toBeInTheDocument();
  });

  it('Run calls onRun when the caller may run this pipeline', async () => {
    const onRun = vi.fn().mockResolvedValue(undefined);
    render(JobsHeader, props({ onRun }));

    await userEvent.click(screen.getByRole('button', { name: 'Run' }));

    expect(onRun).toHaveBeenCalled();
  });

  it('Run is disabled without RUN_PIPELINE, rather than absent', async () => {
    grant([Permission.DELETE_JOB]);
    const onRun = vi.fn();
    render(JobsHeader, props({ onRun }));

    const run = screen.getByRole('button', { name: 'Run' });
    expect(run).toBeDisabled();

    await userEvent.click(run);
    expect(onRun).not.toHaveBeenCalled();
  });

  it('the refresh button calls onRefresh, and says what it is while closed', async () => {
    const onRefresh = vi.fn();
    render(JobsHeader, props({ onRefresh }));

    await userEvent.click(screen.getByRole('button', { name: 'Refresh' }));

    expect(onRefresh).toHaveBeenCalled();
  });

  it('selecting all jobs and deleting them fans the delete out over every selected id', async () => {
    render(JobsHeader, props());

    await userEvent.click(screen.getByRole('button', { name: /select all/i }));
    await userEvent.click(screen.getByRole('button', { name: /delete/i }));
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await vi.waitFor(() => expect(deleteById).toHaveBeenCalledTimes(3));
    expect(deleteById.mock.calls.map(([id]) => id)).toEqual(['job-1', 'job-2', 'job-3']);
  });

  it('the bulk-delete button is disabled without DELETE_JOB', async () => {
    grant([Permission.RUN_PIPELINE]);
    render(JobsHeader, props());

    await userEvent.click(screen.getByRole('button', { name: /select all/i }));

    expect(screen.getByRole('button', { name: /delete/i })).toBeDisabled();
  });
});

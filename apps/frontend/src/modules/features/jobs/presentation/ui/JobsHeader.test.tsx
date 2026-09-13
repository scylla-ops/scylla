import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, act } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { JobsHeader } from './JobsHeader';

const mutateAsyncMock = vi.fn().mockResolvedValue(undefined);
vi.mock('@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts', () => ({
  useDeleteJobs: () => ({ mutateAsync: mutateAsyncMock }),
}));

beforeEach(() => {
  mutateAsyncMock.mockClear();
  useSelectionStore.setState({ selectedIds: {} });
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('JobsHeader', () => {
  it('shows the job count and the pipeline id', () => {
    renderWithI18n(
      <JobsHeader numberOfJobs={3} jobIds={['j1', 'j2', 'j3']} pipelineId='pipeline-9' onRefresh={vi.fn()} />,
    );
    expect(screen.getByText('3')).toBeInTheDocument();
    expect(screen.getByText('Jobs')).toBeInTheDocument();
    expect(screen.getByText(/pipeline-9/)).toBeInTheDocument();
  });

  it('Run calls onRun when the caller may run this pipeline', async () => {
    const onRun = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderWithI18n(
      <JobsHeader numberOfJobs={0} jobIds={[]} pipelineId='pipeline-9' onRefresh={vi.fn()} onRun={onRun} />,
    );

    await user.click(screen.getByRole('button', { name: 'Run' }));
    expect(onRun).toHaveBeenCalled();
  });

  it('Run is disabled without RUN_PIPELINE', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(
      <JobsHeader numberOfJobs={0} jobIds={[]} pipelineId='pipeline-9' onRefresh={vi.fn()} onRun={vi.fn()} />,
    );
    expect(screen.getByRole('button', { name: 'Run' })).toBeDisabled();
  });

  it('the refresh button calls onRefresh', async () => {
    const onRefresh = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <JobsHeader numberOfJobs={2} jobIds={['j1', 'j2']} pipelineId='pipeline-9' onRefresh={onRefresh} />,
    );

    await user.click(screen.getByRole('button', { name: 'Refresh' }));
    expect(onRefresh).toHaveBeenCalled();
  });

  // The bulk-delete button is icon-only (a bare Trash icon) - its "Delete"
  // wording only exists in the tooltip, never as the button's accessible name.
  const getDeleteButton = (): HTMLElement => {
    const button = document.querySelector('button[data-variant="destructive"]');
    if (!button) throw new Error('delete button not found');
    return button as HTMLElement;
  };

  it('selecting all jobs and deleting them fans the delete out over every selected id', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <JobsHeader numberOfJobs={2} jobIds={['j1', 'j2']} pipelineId='pipeline-9' onRefresh={vi.fn()} />,
    );

    await user.click(screen.getByRole('button', { name: 'Select all' }));
    await user.click(getDeleteButton());
    await user.click(screen.getByRole('button', { name: 'Continue' }));

    expect(mutateAsyncMock).toHaveBeenCalledWith('j1');
    expect(mutateAsyncMock).toHaveBeenCalledWith('j2');
    expect(useSelectionStore.getState().selectedIds.jobs ?? []).toEqual([]);
  });

  it('the bulk-delete button is disabled without DELETE_JOB', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <JobsHeader numberOfJobs={2} jobIds={['j1', 'j2']} pipelineId='pipeline-9' onRefresh={vi.fn()} />,
    );
    await user.click(screen.getByRole('button', { name: 'Select all' }));

    // usePermissionsStore is subscribed to directly, so the existing render
    // reacts to this without needing to render again - wrapped in act() since
    // the update happens outside any user-event/React event handler.
    act(() => {
      usePermissionsStore.setState({ permissions: { scopes: [] } });
    });

    expect(getDeleteButton()).toBeDisabled();
  });
});

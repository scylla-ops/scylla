import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { PipelineDashboardHeader } from './PipelineDashboardHeader';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
  useLocation: () => ({ pathname: '/acme/projects/project-1' }),
}));

const mutateAsyncMock = vi.fn().mockResolvedValue(undefined);
vi.mock('@/modules/features/pipeline/presentation/hooks/use-delete-pipeline.ts', () => ({
  useDeletePipeline: () => ({ mutateAsync: mutateAsyncMock }),
}));

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  navigateMock.mockClear();
  mutateAsyncMock.mockClear();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  useSelectionStore.setState({ selectedIds: {} });
  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'web' },
  });
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('PipelineDashboardHeader', () => {
  it('shows the pipeline count', () => {
    renderWithI18n(<PipelineDashboardHeader numberOfPipelines={4} pipelineIds={['p1', 'p2', 'p3', 'p4']} />);
    expect(screen.getByText('4')).toBeInTheDocument();
    expect(screen.getByText('Pipelines')).toBeInTheDocument();
  });

  it('"New pipeline" navigates to the create route under the current project', async () => {
    const user = userEvent.setup();
    renderWithI18n(<PipelineDashboardHeader numberOfPipelines={0} pipelineIds={[]} />);
    await user.click(screen.getByRole('button', { name: 'New pipeline' }));
    expect(navigateMock).toHaveBeenCalledWith('/acme/projects/project-1/create');
  });

  it('"New pipeline" is disabled without CREATE_PIPELINE', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(<PipelineDashboardHeader numberOfPipelines={0} pipelineIds={[]} />);
    expect(screen.getByRole('button', { name: 'New pipeline' })).toBeDisabled();
  });

  it('shows Members/Secrets shortcuts only when permitted, navigating to the sub-route', async () => {
    const user = userEvent.setup();
    renderWithI18n(<PipelineDashboardHeader numberOfPipelines={0} pipelineIds={[]} />);

    await user.click(screen.getByRole('button', { name: 'Members' }));
    expect(navigateMock).toHaveBeenCalledWith('/acme/projects/project-1/members', {});

    await user.click(screen.getByRole('button', { name: 'Secrets' }));
    expect(navigateMock).toHaveBeenCalledWith('/acme/projects/project-1/secrets', {});
  });

  it('hides Members/Secrets without their respective permissions', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(<PipelineDashboardHeader numberOfPipelines={0} pipelineIds={[]} />);
    expect(screen.queryByRole('button', { name: 'Members' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Secrets' })).not.toBeInTheDocument();
  });

  it('selecting all and bulk-deleting fans the delete out over every pipeline id', async () => {
    const user = userEvent.setup();
    renderWithI18n(<PipelineDashboardHeader numberOfPipelines={2} pipelineIds={['p1', 'p2']} />);

    await user.click(screen.getByRole('button', { name: 'Select all' }));
    const deleteButton = document.querySelector('button[data-variant="destructive"]');
    if (!deleteButton) throw new Error('delete button not found');
    await user.click(deleteButton);
    await user.click(screen.getByRole('button', { name: 'Continue' }));

    expect(mutateAsyncMock).toHaveBeenCalledWith('p1');
    expect(mutateAsyncMock).toHaveBeenCalledWith('p2');
  });
});

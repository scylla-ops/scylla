import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { TriggersHeader } from './TriggersHeader';

const mutateAsyncMock = vi.fn().mockResolvedValue(undefined);
vi.mock('@/modules/features/triggers/presentation/hooks/use-delete-trigger.ts', () => ({
  useDeleteTrigger: () => ({ mutateAsync: mutateAsyncMock }),
}));

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  mutateAsyncMock.mockClear();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  useSelectionStore.setState({ selectedIds: {} });
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('TriggersHeader', () => {
  it('shows the trigger count and pipeline id', () => {
    renderWithI18n(
      <TriggersHeader count={2} triggerIds={['t1', 't2']} pipelineId='pipeline-9' onNew={vi.fn()} />,
    );
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('Triggers')).toBeInTheDocument();
    expect(screen.getByText(/pipeline-9/)).toBeInTheDocument();
  });

  it('"New trigger" calls onNew when the caller may manage triggers', async () => {
    const onNew = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<TriggersHeader count={0} triggerIds={[]} pipelineId='p1' onNew={onNew} />);

    await user.click(screen.getByRole('button', { name: 'New trigger' }));
    expect(onNew).toHaveBeenCalled();
  });

  it('"New trigger" is disabled without MANAGE_TRIGGERS', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(<TriggersHeader count={0} triggerIds={[]} pipelineId='p1' onNew={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'New trigger' })).toBeDisabled();
  });

  it('selecting all and bulk-deleting fans the delete out over every id', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <TriggersHeader count={2} triggerIds={['t1', 't2']} pipelineId='p1' onNew={vi.fn()} />,
    );

    await user.click(screen.getByRole('button', { name: 'Select all' }));
    const deleteButton = document.querySelector('button[data-variant="destructive"]');
    if (!deleteButton) throw new Error('delete button not found');
    await user.click(deleteButton);
    await user.click(screen.getByRole('button', { name: 'Continue' }));

    expect(mutateAsyncMock).toHaveBeenCalledWith('t1');
    expect(mutateAsyncMock).toHaveBeenCalledWith('t2');
  });
});

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggersRepository } from '../../../../domain/repository/triggers.repository.ts';
import TriggersHeader from './TriggersHeader.svelte';

const deleteById = vi.fn();

const grant = (permissions: Permission[] | 'all') =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access:
            permissions === 'all' ? { kind: 'fullControl' } : { kind: 'restricted', permissions },
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
    triggers: { triggersRepository: { deleteById } as unknown as TriggersRepository },
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
  count: 2,
  triggerIds: ['trigger-1', 'trigger-2'],
  pipelineId: 'pipeline-1',
  onNew: vi.fn(),
  ...overrides,
});

describe('TriggersHeader', () => {
  it('shows the trigger count and the pipeline it belongs to', () => {
    render(TriggersHeader, props());

    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('pipeline-1')).toBeInTheDocument();
  });

  it('"New trigger" calls onNew when the caller may manage triggers', async () => {
    const onNew = vi.fn();
    render(TriggersHeader, props({ onNew }));

    await userEvent.click(screen.getByRole('button', { name: 'New trigger' }));

    expect(onNew).toHaveBeenCalled();
  });

  it('"New trigger" is disabled without MANAGE_TRIGGERS, rather than absent', async () => {
    grant([Permission.LIST_JOBS]);
    const onNew = vi.fn();
    render(TriggersHeader, props({ onNew }));

    const button = screen.getByRole('button', { name: 'New trigger' });
    expect(button).toBeDisabled();

    await userEvent.click(button);
    expect(onNew).not.toHaveBeenCalled();
  });

  it('deleting a full selection fans the delete out over every selected id', async () => {
    render(TriggersHeader, props());

    await userEvent.click(screen.getByRole('button', { name: /select all/i }));
    await userEvent.click(screen.getByRole('button', { name: /delete/i }));
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await vi.waitFor(() => expect(deleteById).toHaveBeenCalledTimes(2));
    expect(deleteById.mock.calls.map(([id]) => id)).toEqual(['trigger-1', 'trigger-2']);
  });

  it('the bulk-delete button is disabled without MANAGE_TRIGGERS', async () => {
    grant([Permission.LIST_JOBS]);
    render(TriggersHeader, props());

    await userEvent.click(screen.getByRole('button', { name: /select all/i }));

    expect(screen.getByRole('button', { name: /delete/i })).toBeDisabled();
  });

  it('selects the ids that arrived late, not the empty list of the first render', async () => {
    // `createFeatureSelection` takes a getter precisely so that server data
    // arriving after mount is still selectable — capturing the array once
    // froze "select all" on the first, empty render.
    const { rerender } = render(TriggersHeader, props({ count: 0, triggerIds: [] }));

    await rerender(props({ count: 1, triggerIds: ['trigger-9'] }));
    await userEvent.click(screen.getByRole('button', { name: /select all/i }));
    await userEvent.click(screen.getByRole('button', { name: /delete/i }));
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await vi.waitFor(() => expect(deleteById).toHaveBeenCalledWith('trigger-9'));
  });
});

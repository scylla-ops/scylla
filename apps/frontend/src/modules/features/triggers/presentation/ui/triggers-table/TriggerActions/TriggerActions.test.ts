import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { flushSync } from 'svelte';
import { PermissionScope, permissionsStore } from '@platform/authz';
import { findFloating, render } from '@/test/render.svelte.ts';
import TriggerActions from './TriggerActions.svelte';

/** Drives the observer, to reach the narrow layout. */
let notify: ((entries: Array<{ contentRect: { width: number } }>) => void) | null = null;

const narrowTheColumn = async () => {
  notify?.([{ contentRect: { width: 40 } }]);
  flushSync();
  await Promise.resolve();
};

const grantManage = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const handlers = () => ({ onFire: vi.fn(), onEdit: vi.fn(), onDelete: vi.fn() });

beforeEach(() => {
  notify = null;
  vi.stubGlobal(
    'ResizeObserver',
    class {
      constructor(callback: (entries: Array<{ contentRect: { width: number } }>) => void) {
        notify = callback;
      }
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    },
  );
  grantManage();
});

afterEach(() => {
  vi.unstubAllGlobals();
  permissionsStore.setState({ permissions: null });
});

describe('TriggerActions', () => {
  it('offers no action at all without MANAGE_TRIGGERS', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(TriggerActions, { ...handlers() });

    // Svelte leaves an anchor comment: check for no control, not an empty node.
    expect(screen.queryAllByRole('button')).toHaveLength(0);
  });

  it('the wide layout shows fire, edit and delete as three separate buttons', () => {
    render(TriggerActions, { ...handlers() });

    expect(screen.getByRole('button', { name: 'Fire now' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
  });

  it('forwards each wide-layout action to its own callback', async () => {
    const props = handlers();
    render(TriggerActions, props);

    await userEvent.click(screen.getByRole('button', { name: 'Edit' }));

    expect(props.onEdit).toHaveBeenCalled();
    expect(props.onFire).not.toHaveBeenCalled();
    expect(props.onDelete).not.toHaveBeenCalled();
  });

  it('fire now is disabled and marked busy while isFiring is set', () => {
    render(TriggerActions, { ...handlers(), isFiring: true });

    const fireButton = screen.getByRole('button', { name: 'Fire now' });
    expect(fireButton).toBeDisabled();
    expect(fireButton).toHaveAttribute('aria-busy', 'true');
  });

  it('collapses into a single named dropdown trigger once the column narrows', async () => {
    render(TriggerActions, { ...handlers() });
    await narrowTheColumn();

    expect(screen.getByRole('button', { name: 'Trigger actions' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Edit' })).not.toBeInTheDocument();
  });

  it('forwards the action chosen from the compact dropdown', async () => {
    const props = handlers();
    render(TriggerActions, props);
    await narrowTheColumn();

    await userEvent.click(screen.getByRole('button', { name: 'Trigger actions' }));
    await userEvent.click(await findFloating('menuitem', 'Edit'));

    expect(props.onEdit).toHaveBeenCalled();
    expect(props.onFire).not.toHaveBeenCalled();
  });

  it("the dropdown's fire item is disabled while isFiring is set", async () => {
    render(TriggerActions, { ...handlers(), isFiring: true });
    await narrowTheColumn();

    await userEvent.click(screen.getByRole('button', { name: 'Trigger actions' }));

    expect(await findFloating('menuitem', 'Fire now')).toHaveAttribute('data-disabled');
  });
});

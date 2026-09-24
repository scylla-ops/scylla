import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { flushSync } from 'svelte';
import { findFloating, render } from '@/test/render.svelte.ts';
import { PermissionScope, permissionsStore } from '@platform/authz';
import JobActions from './JobActions.svelte';

/**
 * jsdom has no layout engine, so `ResizeObserver` never fires on its own. This
 * double captures the callback the action registers and lets each test drive it
 * with a fabricated width.
 */
class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  callback: (entries: Array<{ contentRect: { width: number } }>) => void;
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();

  constructor(callback: (entries: Array<{ contentRect: { width: number } }>) => void) {
    this.callback = callback;
    ResizeObserverMock.instances.push(this);
  }

  fire(width: number) {
    this.callback([{ contentRect: { width } }]);
    flushSync();
  }
}

const fullControl = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
  fullControl();
});

afterEach(() => permissionsStore.setState({ permissions: null }));

describe('JobActions', () => {
  it('in the wide layout, shows a view action next to delete', () => {
    render(JobActions, { onView: vi.fn(), onDelete: vi.fn() });

    expect(screen.getAllByRole('button')).toHaveLength(2);
    expect(screen.getByRole('button', { name: 'View' })).toBeInTheDocument();
  });

  it('hides the delete button without DELETE_JOB', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(JobActions, { onView: vi.fn(), onDelete: vi.fn() });

    // Only the always-available View button remains.
    expect(screen.getAllByRole('button')).toHaveLength(1);
    expect(screen.getByRole('button', { name: 'View' })).toBeInTheDocument();
  });

  it('collapses into a single, named dropdown trigger once the column is too narrow', () => {
    render(JobActions, { onView: vi.fn(), onDelete: vi.fn() });

    ResizeObserverMock.instances[0].fire(50);

    const buttons = screen.getAllByRole('button');
    expect(buttons).toHaveLength(1);
    // The React trigger was icon-only and unnameable; this one says what it is.
    expect(screen.getByRole('button', { name: 'Job actions' })).toBeInTheDocument();
  });

  it('the collapsed menu forwards a pick to the matching handler and no other', async () => {
    const onView = vi.fn();
    const onDelete = vi.fn();
    render(JobActions, { onView, onDelete });

    ResizeObserverMock.instances[0].fire(50);
    await userEvent.click(screen.getByRole('button', { name: 'Job actions' }));
    await userEvent.click(await findFloating('menuitem', 'View'));

    expect(onView).toHaveBeenCalled();
    expect(onDelete).not.toHaveBeenCalled();
  });

  it('the collapsed menu omits delete without DELETE_JOB', async () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(JobActions, { onView: vi.fn(), onDelete: vi.fn() });

    ResizeObserverMock.instances[0].fire(50);
    await userEvent.click(screen.getByRole('button', { name: 'Job actions' }));

    expect(await findFloating('menuitem', 'View')).toBeInTheDocument();
    expect(screen.queryByText('Delete')).not.toBeInTheDocument();
  });
});

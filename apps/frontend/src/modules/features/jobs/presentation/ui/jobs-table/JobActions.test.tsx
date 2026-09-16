import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { act } from '@testing-library/react';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { JobActions } from './JobActions';

/**
 * jsdom has no real layout engine, so `ResizeObserver` never fires on its own.
 * This double captures the callback the hook registers and lets each test
 * drive it directly with a fabricated `contentRect.width`.
 */
class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  callback: ResizeObserverCallback;
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
    ResizeObserverMock.instances.push(this);
  }

  fire(width: number) {
    this.callback([{ contentRect: { width } } as unknown as ResizeObserverEntry], this);
  }
}

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('JobActions', () => {
  it('in the wide layout, shows a single view action next to delete', () => {
    renderWithI18n(<JobActions onView={vi.fn()} onDelete={vi.fn()} />);
    expect(screen.getAllByRole('button')).toHaveLength(2);
    expect(screen.getByRole('button', { name: 'View' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /logs/i })).not.toBeInTheDocument();
  });

  it('hides the delete button without the matching permission', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(<JobActions onView={vi.fn()} onDelete={vi.fn()} />);
    // Only the always-available View button remains.
    expect(screen.getAllByRole('button')).toHaveLength(1);
  });

  it('collapses into a single dropdown trigger once the container is too narrow', async () => {
    const onView = vi.fn();
    const onDelete = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<JobActions onView={onView} onDelete={onDelete} />);

    act(() => ResizeObserverMock.instances[0].fire(50));

    const trigger = screen.getByRole('button');
    await user.click(trigger);

    await user.click(await screen.findByText('View'));
    expect(onView).toHaveBeenCalled();
    expect(onDelete).not.toHaveBeenCalled();
  });

  it('the dropdown omits the delete item without the matching permission', async () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    const user = userEvent.setup();
    renderWithI18n(<JobActions onView={vi.fn()} onDelete={vi.fn()} />);

    act(() => ResizeObserverMock.instances[0].fire(50));
    await user.click(screen.getByRole('button'));

    expect(await screen.findByText('View')).toBeInTheDocument();
    expect(screen.queryByText('Delete')).not.toBeInTheDocument();
  });
});

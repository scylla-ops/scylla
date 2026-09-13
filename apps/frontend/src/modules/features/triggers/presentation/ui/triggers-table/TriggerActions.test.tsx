import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, act } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { TriggerActions } from './TriggerActions';

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

describe('TriggerActions', () => {
  it('renders nothing at all without MANAGE_TRIGGERS', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    const { container } = renderWithI18n(
      <TriggerActions onFire={vi.fn()} onEdit={vi.fn()} onDelete={vi.fn()} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('the wide layout shows three separate icon buttons', () => {
    renderWithI18n(<TriggerActions onFire={vi.fn()} onEdit={vi.fn()} onDelete={vi.fn()} />);
    expect(screen.getAllByRole('button')).toHaveLength(3);
  });

  it('fire now is disabled and marked busy while isFiring is set', () => {
    renderWithI18n(
      <TriggerActions onFire={vi.fn()} onEdit={vi.fn()} onDelete={vi.fn()} isFiring />,
    );
    const [fireButton] = screen.getAllByRole('button');
    expect(fireButton).toBeDisabled();
    expect(fireButton).toHaveAttribute('aria-busy', 'true');
  });

  it('collapses into a dropdown once the container narrows, and forwards each action', async () => {
    const onFire = vi.fn();
    const onEdit = vi.fn();
    const onDelete = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<TriggerActions onFire={onFire} onEdit={onEdit} onDelete={onDelete} />);

    act(() => ResizeObserverMock.instances[0].fire(50));
    await user.click(screen.getByRole('button'));

    await user.click(await screen.findByText('Edit'));
    expect(onEdit).toHaveBeenCalled();
    expect(onFire).not.toHaveBeenCalled();
    expect(onDelete).not.toHaveBeenCalled();
  });

  it('the dropdown\'s "Fire now" item is disabled while isFiring is set', async () => {
    const user = userEvent.setup();
    renderWithI18n(<TriggerActions onFire={vi.fn()} onEdit={vi.fn()} onDelete={vi.fn()} isFiring />);

    act(() => ResizeObserverMock.instances[0].fire(50));
    await user.click(screen.getByRole('button'));

    const fireItem = (await screen.findByText('Fire now')).closest('[role="menuitem"]');
    expect(fireItem).toHaveAttribute('data-disabled');
  });
});

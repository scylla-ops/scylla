import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, act } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { PipelineActions } from './PipelineActions';

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

describe('PipelineActions', () => {
  it('the wide layout shows Run/Edit/Duplicate when permitted, plus optional Jobs/Triggers/More', () => {
    renderWithI18n(
      <PipelineActions
        onRun={vi.fn()}
        onEdit={vi.fn()}
        onDuplicate={vi.fn()}
        onViewJobs={vi.fn()}
        onViewTriggers={vi.fn()}
        onMore={vi.fn()}
      />,
    );
    expect(screen.getAllByRole('button')).toHaveLength(6);
  });

  it('omits Run/Edit/Duplicate without their respective permissions', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(<PipelineActions onRun={vi.fn()} onEdit={vi.fn()} onDuplicate={vi.fn()} />);
    expect(screen.queryAllByRole('button')).toHaveLength(0);
  });

  it('omits the Triggers action without MANAGE_TRIGGERS even when onViewTriggers is given', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(
      <PipelineActions onRun={vi.fn()} onEdit={vi.fn()} onDuplicate={vi.fn()} onViewJobs={vi.fn()} onViewTriggers={vi.fn()} />,
    );
    // Only View Jobs doesn't need a permission of its own.
    expect(screen.getAllByRole('button')).toHaveLength(1);
  });

  it('Run is disabled (and spinning) while isRunning', () => {
    renderWithI18n(
      <PipelineActions onRun={vi.fn()} onEdit={vi.fn()} onDuplicate={vi.fn()} isRunning />,
    );
    const [runButton] = screen.getAllByRole('button');
    expect(runButton).toBeDisabled();
    expect(runButton).toHaveAttribute('aria-busy', 'true');
  });

  it('collapses into a dropdown once the container narrows, and forwards each action', async () => {
    const onEdit = vi.fn();
    const onRun = vi.fn();
    const onDuplicate = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<PipelineActions onRun={onRun} onEdit={onEdit} onDuplicate={onDuplicate} />);

    act(() => ResizeObserverMock.instances[0].fire(50));
    await user.click(screen.getByRole('button'));

    await user.click(await screen.findByText('Edit'));
    expect(onEdit).toHaveBeenCalled();
    expect(onRun).not.toHaveBeenCalled();
    expect(onDuplicate).not.toHaveBeenCalled();
  });
});

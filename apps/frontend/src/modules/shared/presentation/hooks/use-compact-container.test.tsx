import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act, render, screen } from '@testing-library/react';
import { useCompactContainer } from './use-compact-container';

/**
 * jsdom has no real layout engine, so `ResizeObserver` never fires on its own.
 * This double captures the callback the hook registers and lets each test
 * drive it directly with a fabricated `contentRect.width` - the same
 * controllable-double idiom used elsewhere in this suite for async streams.
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
    this.callback(
      [{ contentRect: { width } } as unknown as ResizeObserverEntry],
      this,
    );
  }
}

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
});

const TestComponent = ({ threshold }: { threshold?: number }) => {
  const { containerRef, isCompact } = useCompactContainer(threshold);
  return <div ref={containerRef}>{isCompact ? 'compact' : 'wide'}</div>;
};

describe('useCompactContainer', () => {
  it('starts wide before any measurement has come in', () => {
    render(<TestComponent />);
    expect(screen.getByText('wide')).toBeInTheDocument();
  });

  it('flips to compact once the observed width drops under the default 70px threshold', () => {
    render(<TestComponent />);
    const [observer] = ResizeObserverMock.instances;

    act(() => observer.fire(50));
    expect(screen.getByText('compact')).toBeInTheDocument();

    act(() => observer.fire(100));
    expect(screen.getByText('wide')).toBeInTheDocument();
  });

  it('honors a custom threshold instead of the 70px default', () => {
    render(<TestComponent threshold={200} />);
    const [observer] = ResizeObserverMock.instances;

    act(() => observer.fire(150)); // wide by default, compact at this threshold
    expect(screen.getByText('compact')).toBeInTheDocument();
  });

  it('disconnects the observer on unmount', () => {
    const { unmount } = render(<TestComponent />);
    const [observer] = ResizeObserverMock.instances;

    unmount();

    expect(observer.disconnect).toHaveBeenCalled();
  });
});

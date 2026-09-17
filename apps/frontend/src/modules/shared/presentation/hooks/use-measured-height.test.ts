import { describe, it, expect, beforeEach, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useMeasuredHeight } from './use-measured-height';

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

  fire(height: number) {
    this.callback([{ contentRect: { height } } as unknown as ResizeObserverEntry], this);
  }
}

beforeEach(() => {
  ResizeObserverMock.instances = [];
  vi.stubGlobal('ResizeObserver', ResizeObserverMock);
});

const divWithHeight = (height: number) => {
  const el = document.createElement('div');
  Object.defineProperty(el, 'clientHeight', { configurable: true, value: height });
  return el;
};

describe('useMeasuredHeight', () => {
  it('has no height to report before a container is attached', () => {
    const { result } = renderHook(() => useMeasuredHeight());

    expect(result.current.height).toBeNull();
    expect(ResizeObserverMock.instances).toHaveLength(0);
  });

  it('measures on attach, without waiting for the observer to report', () => {
    const { result } = renderHook(() => useMeasuredHeight());

    act(() => result.current.containerRef(divWithHeight(640)));

    expect(result.current.height).toBe(640);
  });

  it('reports every size the observer sees afterwards', () => {
    const { result } = renderHook(() => useMeasuredHeight());
    act(() => result.current.containerRef(divWithHeight(640)));

    act(() => ResizeObserverMock.instances[0].fire(900));

    expect(result.current.height).toBe(900);
  });

  it('observes an element that only mounts later', () => {
    const { result } = renderHook(() => useMeasuredHeight());

    act(() => result.current.containerRef(null));
    expect(ResizeObserverMock.instances).toHaveLength(0);

    act(() => result.current.containerRef(divWithHeight(480)));
    expect(ResizeObserverMock.instances).toHaveLength(1);
    expect(result.current.height).toBe(480);
  });

  it('keeps the last known height when the element detaches, and stops observing', () => {
    const { result } = renderHook(() => useMeasuredHeight());
    act(() => result.current.containerRef(divWithHeight(640)));
    const observer = ResizeObserverMock.instances[0];

    act(() => result.current.containerRef(null));

    expect(result.current.height).toBe(640);
    expect(observer.disconnect).toHaveBeenCalled();
  });
});

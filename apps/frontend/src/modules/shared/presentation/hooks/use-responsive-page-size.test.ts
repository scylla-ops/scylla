import { describe, it, expect, beforeEach, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useResponsivePageSize } from './use-responsive-page-size';

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

describe('useResponsivePageSize', () => {
  it('starts at the minimum before any container is attached', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    expect(result.current.pageSize).toBe(5);
  });

  it('reports itself unmeasured until a container is attached', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    expect(result.current.isMeasured).toBe(false);

    act(() => result.current.containerRef(divWithHeight(640)));
    expect(result.current.isMeasured).toBe(true);
  });

  it('stays measured when the container detaches, keeping the last known size', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(divWithHeight(640)));
    act(() => result.current.containerRef(null));

    expect(result.current.isMeasured).toBe(true);
    expect(result.current.pageSize).toBe(10);
  });

  it('computes the page size synchronously on attach, without waiting for the observer', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(divWithHeight(640)));
    expect(result.current.pageSize).toBe(10);
  });

  it('starts observing once the containerRef callback is given a real element', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(document.createElement('div')));
    expect(ResizeObserverMock.instances).toHaveLength(1);
  });

  it('derives the page size from the container height, row height and header height', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(document.createElement('div')));
    act(() => ResizeObserverMock.instances[0].fire(640));
    expect(result.current.pageSize).toBe(10);
  });

  it('clamps to the minimum for a very short container', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(document.createElement('div')));
    act(() => ResizeObserverMock.instances[0].fire(80));
    expect(result.current.pageSize).toBe(5);
  });

  it('clamps to the maximum for a very tall container', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(document.createElement('div')));
    act(() => ResizeObserverMock.instances[0].fire(10000));
    expect(result.current.pageSize).toBe(50);
  });

  it('recomputes whenever the observer reports a new size', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(document.createElement('div')));
    act(() => ResizeObserverMock.instances[0].fire(640));
    expect(result.current.pageSize).toBe(10);

    act(() => ResizeObserverMock.instances[0].fire(1240));
    expect(result.current.pageSize).toBe(20);
  });

  it('re-observes when the element mounts after an initial null attach (the loading-placeholder case)', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));

    act(() => result.current.containerRef(null));
    expect(ResizeObserverMock.instances).toHaveLength(0);

    act(() => result.current.containerRef(document.createElement('div')));
    expect(ResizeObserverMock.instances).toHaveLength(1);

    act(() => ResizeObserverMock.instances[0].fire(640));
    expect(result.current.pageSize).toBe(10);
  });

  it('disconnects the observer when the element detaches', () => {
    const { result } = renderHook(() => useResponsivePageSize({ rowHeight: 60, headerHeight: 40 }));
    act(() => result.current.containerRef(document.createElement('div')));
    const observer = ResizeObserverMock.instances[0];

    act(() => result.current.containerRef(null));
    expect(observer.disconnect).toHaveBeenCalled();
  });
});

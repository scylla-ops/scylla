import { describe, it, expect, vi, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useNow } from './use-now';

describe('useNow', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns the current timestamp on first render', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:00.000Z'));
    const { result } = renderHook(() => useNow());
    expect(result.current).toBe(new Date('2026-01-01T00:00:00.000Z').getTime());
  });

  it('ticks forward on the given interval while enabled', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:00.000Z'));
    const { result } = renderHook(() => useNow(true, 1000));

    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(result.current).toBe(new Date('2026-01-01T00:00:01.000Z').getTime());

    act(() => {
      vi.advanceTimersByTime(2000);
    });
    expect(result.current).toBe(new Date('2026-01-01T00:00:03.000Z').getTime());
  });

  it('does not tick when disabled', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:00.000Z'));
    const { result } = renderHook(() => useNow(false, 1000));
    const initial = result.current;

    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(result.current).toBe(initial);
  });

  it('clears its interval on unmount (no dangling timer)', () => {
    vi.useFakeTimers();
    const clearSpy = vi.spyOn(window, 'clearInterval');
    const { unmount } = renderHook(() => useNow(true, 1000));
    unmount();
    expect(clearSpy).toHaveBeenCalled();
  });
});

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useCodeMirrorTheme } from './use-code-mirror-theme';
import { setTheme } from '@shared/presentation/stores/theme.store.ts';

const buildCodeMirrorTheme = vi.fn().mockReturnValue('fake-extension');
vi.mock('@shared/presentation/utils/code-mirror-theme.ts', () => ({
  buildCodeMirrorTheme: (...args: unknown[]) => buildCodeMirrorTheme(...args),
}));

// The real store, not a mocked `useTheme`: the rule under test is how the hook
// reads the app theme, and mocking that away would leave nothing to check.
beforeEach(() => {
  setTheme('dark');
  buildCodeMirrorTheme.mockClear();
});

describe('useCodeMirrorTheme', () => {
  it('builds the dark theme when the app theme is dark', () => {
    renderHook(() => useCodeMirrorTheme());

    expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: true, hasError: false });
  });

  it('builds the light theme only for the exact theme "light"', () => {
    setTheme('light');

    renderHook(() => useCodeMirrorTheme());

    expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: false, hasError: false });
  });

  it('forwards hasError unchanged', () => {
    setTheme('light');

    renderHook(() => useCodeMirrorTheme({ hasError: true }));

    expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: false, hasError: true });
  });

  it('memoizes the extension: an unrelated re-render does not rebuild it', () => {
    const { result, rerender } = renderHook(() => useCodeMirrorTheme());
    const first = result.current;

    rerender();

    expect(result.current).toBe(first);
    expect(buildCodeMirrorTheme).toHaveBeenCalledTimes(1);
  });

  it('rebuilds the extension when hasError changes', () => {
    const { rerender } = renderHook(
      ({ hasError }: { hasError: boolean }) => useCodeMirrorTheme({ hasError }),
      { initialProps: { hasError: false } },
    );

    rerender({ hasError: true });

    expect(buildCodeMirrorTheme).toHaveBeenCalledTimes(2);
  });

  it('rebuilds the extension when the app theme changes under it', () => {
    const { rerender } = renderHook(() => useCodeMirrorTheme());

    setTheme('light');
    rerender();

    expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: false, hasError: false });
  });
});

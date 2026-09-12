import { describe, it, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useCodeMirrorTheme } from './use-code-mirror-theme';

let resolvedThemeFixture: string | undefined = 'dark';
vi.mock('next-themes', () => ({
  useTheme: () => ({ resolvedTheme: resolvedThemeFixture }),
}));

const buildCodeMirrorTheme = vi.fn().mockReturnValue('fake-extension');
vi.mock('@shared/presentation/utils/code-mirror-theme.ts', () => ({
  buildCodeMirrorTheme: (...args: unknown[]) => buildCodeMirrorTheme(...args),
}));

describe('useCodeMirrorTheme', () => {
  it('is dark whenever resolvedTheme is anything other than exactly "light"', () => {
    for (const resolvedTheme of ['dark', undefined, 'system', 'anything-else']) {
      resolvedThemeFixture = resolvedTheme;
      renderHook(() => useCodeMirrorTheme());
      expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: true, hasError: false });
    }
  });

  it('is light only for the exact string "light"', () => {
    resolvedThemeFixture = 'light';
    renderHook(() => useCodeMirrorTheme());
    expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: false, hasError: false });
  });

  it('forwards hasError unchanged', () => {
    resolvedThemeFixture = 'light';
    renderHook(() => useCodeMirrorTheme({ hasError: true }));
    expect(buildCodeMirrorTheme).toHaveBeenLastCalledWith({ isDark: false, hasError: true });
  });

  it('memoizes the extension: an unrelated re-render does not rebuild it', () => {
    resolvedThemeFixture = 'light';
    buildCodeMirrorTheme.mockClear();
    const { result, rerender } = renderHook(() => useCodeMirrorTheme());
    const first = result.current;

    rerender();

    expect(result.current).toBe(first);
    expect(buildCodeMirrorTheme).toHaveBeenCalledTimes(1);
  });

  it('rebuilds the extension when hasError changes', () => {
    resolvedThemeFixture = 'light';
    buildCodeMirrorTheme.mockClear();
    const { rerender } = renderHook(
      ({ hasError }: { hasError: boolean }) => useCodeMirrorTheme({ hasError }),
      { initialProps: { hasError: false } },
    );

    rerender({ hasError: true });

    expect(buildCodeMirrorTheme).toHaveBeenCalledTimes(2);
  });
});

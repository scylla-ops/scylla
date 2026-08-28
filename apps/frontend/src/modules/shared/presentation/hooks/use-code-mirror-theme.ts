import { useMemo } from 'react';
import { useTheme } from 'next-themes';
import type { Extension } from '@uiw/react-codemirror';
import { buildCodeMirrorTheme } from '@shared/presentation/utils/code-mirror-theme.ts';

interface UseCodeMirrorThemeOptions {
  /** Renders the editor in destructive/error colors (e.g. invalid script). */
  hasError?: boolean;
}

/**
 * CodeMirror theme bound to the app's color scheme. Pass the result to the
 * `theme` prop of `<ReactCodeMirror />` so the editor surface, gutters and
 * token colors follow light/dark like the rest of the app.
 */
export const useCodeMirrorTheme = ({ hasError = false }: UseCodeMirrorThemeOptions = {}) => {
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme !== 'light';

  return useMemo<Extension>(() => buildCodeMirrorTheme({ isDark, hasError }), [isDark, hasError]);
};
